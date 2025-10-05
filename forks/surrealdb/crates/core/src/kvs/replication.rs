use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use prometheus::{Registry, IntGauge, IntCounter};
use tokio::sync::{broadcast, RwLock};

#[derive(Debug)]
pub struct ReplicaMonitor {
	pub replica_id: String,
	lag_seconds: IntGauge,
	lag_operations: IntCounter,
	last_update: AtomicU64,
	alert_threshold_secs: u64,
}

impl ReplicaMonitor {
	pub fn new(
		replica_id: String,
		registry: &Registry,
		alert_threshold_secs: u64,
	) -> Result<Self, prometheus::Error> {
		let lag_seconds = IntGauge::new(
			format!("surrealdb_replication_lag_seconds_{}", replica_id),
			"Replication lag in seconds",
		)?;

		let lag_operations = IntCounter::new(
			format!("surrealdb_replication_lag_operations_{}", replica_id),
			"Number of operations behind primary",
		)?;

		registry.register(Box::new(lag_seconds.clone()))?;
		registry.register(Box::new(lag_operations.clone()))?;

		Ok(Self {
			replica_id,
			lag_seconds,
			lag_operations,
			last_update: AtomicU64::new(0),
			alert_threshold_secs,
		})
	}

	pub fn update(&self, timestamp: SystemTime) {
		let ts = timestamp
			.duration_since(UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0);
		self.last_update.store(ts, Ordering::Relaxed);
	}

	pub fn calculate_lag(&self) -> Duration {
		let now = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0);

		let last = self.last_update.load(Ordering::Relaxed);
		let lag = now.saturating_sub(last);

		self.lag_seconds.set(lag as i64);

		Duration::from_secs(lag)
	}

	pub fn is_lagging(&self) -> bool {
		let lag = self.calculate_lag().as_secs();
		lag > self.alert_threshold_secs
	}

	pub fn increment_operations(&self) {
		self.lag_operations.inc();
	}
}

#[derive(Clone, Debug)]
pub enum ReplicationEvent {
	LagAlert {
		replica_id: String,
		lag_seconds: u64,
		threshold: u64,
	},
	LagResolved {
		replica_id: String,
		lag_seconds: u64,
	},
}

pub struct ReplicationManager {
	replicas: Arc<RwLock<HashMap<String, Arc<ReplicaMonitor>>>>,
	event_tx: broadcast::Sender<ReplicationEvent>,
	registry: Arc<Registry>,
}

impl ReplicationManager {
	pub fn new(registry: Arc<Registry>) -> Self {
		let (tx, _) = broadcast::channel(100);
		Self {
			replicas: Arc::new(RwLock::new(HashMap::new())),
			event_tx: tx,
			registry,
		}
	}

	pub async fn add_replica(&self, replica_id: String, threshold_secs: u64) -> Result<(), prometheus::Error> {
		let monitor = Arc::new(ReplicaMonitor::new(
			replica_id.clone(),
			&self.registry,
			threshold_secs,
		)?);

		self.replicas.write().await.insert(replica_id, monitor);
		Ok(())
	}

	pub async fn get_monitor(&self, replica_id: &str) -> Option<Arc<ReplicaMonitor>> {
		self.replicas.read().await.get(replica_id).cloned()
	}

	pub async fn run_monitor_loop(&self) {
		let mut interval = tokio::time::interval(Duration::from_secs(1));
		let mut previous_states: HashMap<String, bool> = HashMap::new();

		loop {
			interval.tick().await;

			let replicas = self.replicas.read().await.clone();
			for (id, monitor) in replicas.iter() {
				let lag = monitor.calculate_lag();
				let is_lagging = monitor.is_lagging();

				let was_lagging = previous_states.get(id).copied().unwrap_or(false);

				if is_lagging && !was_lagging {
					let _ = self.event_tx.send(ReplicationEvent::LagAlert {
						replica_id: id.clone(),
						lag_seconds: lag.as_secs(),
						threshold: monitor.alert_threshold_secs,
					});
				} else if !is_lagging && was_lagging {
					let _ = self.event_tx.send(ReplicationEvent::LagResolved {
						replica_id: id.clone(),
						lag_seconds: lag.as_secs(),
					});
				}

				previous_states.insert(id.clone(), is_lagging);
			}
		}
	}

	pub fn subscribe(&self) -> broadcast::Receiver<ReplicationEvent> {
		self.event_tx.subscribe()
	}
}
