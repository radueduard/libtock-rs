use core::cell::Cell;
use libtock_platform::{share, ErrorCode, Syscalls, YieldNoWaitReturn};
use libtock_unittest::fake;

type Humidity = super::Humidity<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert_eq!(Humidity::exists(), Err(ErrorCode::NoDevice));
}

#[test]
fn driver_check() {
    let kernel = fake::Kernel::new();
    let driver = fake::Humidity::new();
    kernel.add_driver(&driver);

    assert_eq!(Humidity::exists(), Ok(()));
}

#[test]
fn humidity_read() {
    let kernel = fake::Kernel::new();
    let driver = fake::Humidity::new();
    kernel.add_driver(&driver);

    assert_eq!(Humidity::humidity_read(), Ok(()));
    assert!(driver.is_busy());

    assert_eq!(Humidity::humidity_read(), Err(ErrorCode::Busy));
    assert_eq!(Humidity::humidity_read_sync(), Err(ErrorCode::Busy));
}

#[test]
fn register_unregister_listener() {
    let kernel = fake::Kernel::new();
    let driver = fake::Humidity::new();
    kernel.add_driver(&driver);

    let humidity_cell: Cell<Option<i32>> = Cell::new(None);
    let listener = crate::HumidityListener(|temp_val| {
        humidity_cell.set(Some(temp_val));
    });
    share::scope(|subscribe| {
        assert_eq!(Humidity::humidity_read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert_eq!(Humidity::register_listener(&listener, subscribe), Ok(()));
        assert_eq!(Humidity::humidity_read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(humidity_cell.get(), Some(100));

        Humidity::unregister_listener();
        assert_eq!(Humidity::humidity_read(), Ok(()));
        driver.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);
    });
}

#[test]
fn humidity_read_sync() {
    let kernel = fake::Kernel::new();
    let driver = fake::Humidity::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(1000);
    assert_eq!(Humidity::humidity_read_sync(), Ok(1000));
}

#[test]
fn negative_value() {
    let kernel = fake::Kernel::new();
    let driver = fake::Humidity::new();
    kernel.add_driver(&driver);

    driver.set_value_sync(-1000);
    assert_eq!(Humidity::humidity_read_sync(), Ok(-1000));
}
