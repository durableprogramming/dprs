use super::*;
use std::thread::sleep;

#[test]
fn test_toast_creation() {
    let toast = Toast::new("Test message", 100);
    assert_eq!(toast.message, "Test message");
    assert_eq!(toast.duration, Duration::from_millis(100));
}

#[test]
fn test_toast_expiration() {
    let toast = Toast::new("Test message", 10);
    sleep(Duration::from_millis(20));
    assert!(toast.is_expired());
}

#[test]
fn test_toast_not_expired() {
    let toast = Toast::new("Test message", 1000);
    assert!(!toast.is_expired());
}

#[test]
fn test_toast_manager_show() {
    let mut manager = ToastManager::new();
    assert!(manager.get_toast().is_none());
    
    manager.show("New toast", 100);
    let toast = manager.get_toast().unwrap();
    assert_eq!(toast.message, "New toast");
}

#[test]
fn test_toast_manager_clear() {
    let mut manager = ToastManager::new();
    manager.show("Test toast", 100);
    assert!(manager.get_toast().is_some());
    
    manager.clear();
    assert!(manager.get_toast().is_none());
}

#[test]
fn test_toast_manager_check_expired() {
    let mut manager = ToastManager::new();
    manager.show("Expiring toast", 10);
    assert!(manager.get_toast().is_some());
    
    sleep(Duration::from_millis(20));
    manager.check_expired();
    assert!(manager.get_toast().is_none());
}

#[test]
fn test_toast_manager_not_expired() {
    let mut manager = ToastManager::new();
    manager.show("Non-expiring toast", 100);
    
    manager.check_expired();
    assert!(manager.get_toast().is_some());
}

#[test]
fn test_toast_manager_default() {
    let manager = ToastManager::default();
    assert!(manager.get_toast().is_none());
}
