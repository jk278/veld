//! UI Component Library
//! 可复用 UI 组件库 - 统一样式和交互模式

pub mod badge;
pub mod button;
pub mod card;
pub mod input;
pub mod modal;
pub mod tabs;

// Re-export commonly used components
pub use badge::{Badge, BadgeVariant, ProviderBadge, StatusBadge, StatusType, Tag};
pub use button::{Button, ButtonVariant, CancelButton, PrimaryButton, SecondaryButton};
pub use card::{
  Card, CardContent, CardFooter, CardHeader, InfoCard, InfoCardVariant, ListItem, StaticCard,
};
pub use input::{PasswordField, TextArea, TextField};
pub use modal::{AdvancedSection, FormSection, Modal, ModalContent, ModalFooter, ModalHeader};
pub use tabs::{NavTab, Tab, TabList, TabPanel, Tabs};
