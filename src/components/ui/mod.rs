//! UI Component Library
//! 可复用 UI 组件库 - 统一样式和交互模式

pub mod button;
pub mod input;
pub mod modal;
pub mod tabs;
pub mod card;
pub mod badge;

// Re-export commonly used components
pub use button::{Button, ButtonVariant, PrimaryButton, SecondaryButton, CancelButton};
pub use input::{TextField, TextArea, PasswordField};
pub use modal::{Modal, ModalHeader, ModalContent, ModalFooter, FormSection, AdvancedSection};
pub use tabs::{Tabs, TabList, Tab, TabPanel, NavTab};
pub use card::{Card, StaticCard, CardHeader, CardContent, CardFooter, InfoCard, InfoCardVariant, ListItem};
pub use badge::{Badge, BadgeVariant, StatusBadge, StatusType, Tag, ProviderBadge};
