use crate::fake::{self, SyscallDriver};
use fake::humidity::*;
use libtock_platform::{share, DefaultConfig, YieldNoWaitReturn};

//Test the command implementation
#[test]
fn command() {
    let hum = Humidity::new();

    assert!(hum.command(EXISTS, 1, 2).is_success());

    assert!(hum.command(READ_HUM, 0, 0).is_success());

    assert_eq!(
        hum.command(READ_HUM, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );

    hum.set_value(100);
    assert!(hum.command(READ_HUM, 0, 1).is_success());
    hum.set_value(100);

    hum.set_value_sync(100);
    assert!(hum.command(READ_HUM, 0, 1).is_success());
    assert!(hum.command(READ_HUM, 0, 1).is_success());
}

// Integration test that verifies Humidity works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let hum = Humidity::new();
    kernel.add_driver(&hum);
    assert!(fake::Syscalls::command(DRIVER_NUM, EXISTS, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_HUM, 0, 0).is_success());
    assert_eq!(
        fake::Syscalls::command(DRIVER_NUM, READ_HUM, 0, 0).get_failure(),
        Some(ErrorCode::Busy)
    );
    hum.set_value(100);
    assert!(fake::Syscalls::command(DRIVER_NUM, READ_HUM, 0, 1).is_success());

    let listener = Cell::<Option<(u32,)>>::new(None);
    share::scope(|subscribe| {
        assert_eq!(
            fake::Syscalls::subscribe::<_, _, DefaultConfig, DRIVER_NUM, 0>(subscribe, &listener),
            Ok(())
        );

        hum.set_value(100);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
        assert_eq!(listener.get(), Some((100,)));

        hum.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::NoUpcall);

        assert!(fake::Syscalls::command(DRIVER_NUM, READ_HUM, 0, 1).is_success());
        hum.set_value(200);
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);

        hum.set_value_sync(200);
        assert!(fake::Syscalls::command(DRIVER_NUM, READ_HUM, 0, 1).is_success());
        assert_eq!(fake::Syscalls::yield_no_wait(), YieldNoWaitReturn::Upcall);
    });
}
