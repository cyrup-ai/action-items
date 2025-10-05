# objc2-core-bluetooth API Analysis

Based on analysis of the objc2-core-bluetooth framework source, the CBCentralManager API has changed:

## Key Findings

1. **CBCentralManager::authorization()** - This is a static method that does NOT take MainThreadMarker
2. **CBCentralManager::new()** - This takes MainThreadMarker as parameter
3. **The ivars() method issue** - The define_class! macro requires proper impl block with init method

## Current API Patterns

From the objc2 source analysis:

```rust
// Static authorization method - no MainThreadMarker needed
let auth = unsafe { CBCentralManager::authorization() };

// Instance creation requires MainThreadMarker
let manager = CBCentralManager::new(mtm);

// Delegate setting
manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
```

## define_class! Pattern

The correct pattern for delegates with ivars requires:

```rust
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = DelegateIvars]
    struct MyDelegate;

    impl MyDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(DelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl SomeDelegate for MyDelegate {
        #[unsafe(method(delegateMethod))]
        fn delegate_method(&self) {
            // self.ivars() is now available
        }
    }
);
```

The key is that the impl block with init method MUST be inside the define_class! macro for ivars() to be generated.