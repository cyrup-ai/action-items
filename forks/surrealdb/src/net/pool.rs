use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Semaphore, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PoolConfig {
	pub min_size: usize,
	pub max_size: usize,
	pub initial_size: usize,
	pub scale_up_threshold: f64,
	pub scale_down_threshold: f64,
	pub check_interval: Duration,
	pub scale_cooldown: Duration,
}

impl Default for PoolConfig {
	fn default() -> Self {
		Self {
			min_size: 2,
			max_size: 20,
			initial_size: 5,
			scale_up_threshold: 0.75,
			scale_down_threshold: 0.25,
			check_interval: Duration::from_secs(10),
			scale_cooldown: Duration::from_secs(30),
		}
	}
}

#[derive(Debug)]
struct PoolMetrics {
	active_connections: AtomicUsize,
	total_wait_time: AtomicUsize,
	wait_count: AtomicUsize,
	last_scale_time: RwLock<Instant>,
}

impl Default for PoolMetrics {
	fn default() -> Self {
		Self {
			active_connections: AtomicUsize::new(0),
			total_wait_time: AtomicUsize::new(0),
			wait_count: AtomicUsize::new(0),
			last_scale_time: RwLock::new(Instant::now()),
		}
	}
}

impl PoolMetrics {
	fn utilization(&self, pool_size: usize) -> f64 {
		let active = self.active_connections.load(Ordering::Relaxed);
		if pool_size == 0 {
			return 0.0;
		}
		active as f64 / pool_size as f64
	}

	fn avg_wait_time(&self) -> Duration {
		let total = self.total_wait_time.load(Ordering::Relaxed);
		let count = self.wait_count.load(Ordering::Relaxed);

		if count == 0 {
			Duration::ZERO
		} else {
			Duration::from_millis((total / count) as u64)
		}
	}

	fn record_wait(&self, duration: Duration) {
		self.total_wait_time.fetch_add(
			duration.as_millis() as usize,
			Ordering::Relaxed,
		);
		self.wait_count.fetch_add(1, Ordering::Relaxed);
	}
}

pub struct AdaptivePool<T> {
	connections: Arc<RwLock<Vec<T>>>,
	semaphore: Arc<Semaphore>,
	current_size: Arc<AtomicUsize>,
	metrics: Arc<PoolMetrics>,
	config: PoolConfig,
}

impl<T> AdaptivePool<T> {
	pub async fn new<F>(config: PoolConfig, factory: F) -> Self
	where
		F: Fn() -> T,
	{
		let mut connections = Vec::new();
		for _ in 0..config.initial_size {
			connections.push(factory());
		}

		let semaphore = Arc::new(Semaphore::new(config.initial_size));

		let pool = Self {
			connections: Arc::new(RwLock::new(connections)),
			semaphore,
			current_size: Arc::new(AtomicUsize::new(config.initial_size)),
			metrics: Arc::new(PoolMetrics::default()),
			config,
		};

		pool.spawn_monitor();

		pool
	}

	pub async fn get(&self) -> PoolConnection<T> {
		let start = Instant::now();

		let permit = self
			.semaphore
			.clone()
			.acquire_owned()
			.await
			.map_err(|_| "Failed to acquire semaphore permit")
			.ok();

		let wait = start.elapsed();
		if wait > Duration::from_millis(10) {
			self.metrics.record_wait(wait);
		}

		self.metrics
			.active_connections
			.fetch_add(1, Ordering::Relaxed);

		PoolConnection {
			_permit: permit,
			metrics: self.metrics.clone(),
			_phantom: PhantomData,
		}
	}

	fn spawn_monitor(&self) {
		let metrics = self.metrics.clone();
		let semaphore = self.semaphore.clone();
		let current_size = self.current_size.clone();
		let config = self.config.clone();

		tokio::spawn(async move {
			let mut interval = tokio::time::interval(config.check_interval);

			loop {
				interval.tick().await;

				let size = current_size.load(Ordering::Relaxed);
				let util = metrics.utilization(size);
				let avg_wait = metrics.avg_wait_time();

				let last_scale = *metrics.last_scale_time.read().await;
				if last_scale.elapsed() < config.scale_cooldown {
					continue;
				}

				if (util > config.scale_up_threshold || avg_wait > Duration::from_millis(50))
					&& size < config.max_size
				{
					let new_size = (size + 2).min(config.max_size);
					semaphore.add_permits(new_size - size);
					current_size.store(new_size, Ordering::Relaxed);
					*metrics.last_scale_time.write().await = Instant::now();

					eprintln!("Pool scaled UP: {} -> {}", size, new_size);
				} else if util < config.scale_down_threshold && size > config.min_size {
					let new_size = (size - 1).max(config.min_size);
					current_size.store(new_size, Ordering::Relaxed);
					*metrics.last_scale_time.write().await = Instant::now();

					eprintln!("Pool scaled DOWN: {} -> {}", size, new_size);
				}
			}
		});
	}
}

pub struct PoolConnection<T> {
	_permit: Option<tokio::sync::OwnedSemaphorePermit>,
	metrics: Arc<PoolMetrics>,
	_phantom: PhantomData<T>,
}

impl<T> Drop for PoolConnection<T> {
	fn drop(&mut self) {
		self.metrics
			.active_connections
			.fetch_sub(1, Ordering::Relaxed);
	}
}
