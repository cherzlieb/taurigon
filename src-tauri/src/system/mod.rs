//! # System-Modul
//!
//! Kapselt alles rund um die Erkennung und Analyse der Laufzeitumgebung:
//! Container-Engine, Distribution, SELinux-Status.
//!
//! Dieses Modul ist rein lesend – es verändert den Systemzustand nicht.

pub mod inspector;
