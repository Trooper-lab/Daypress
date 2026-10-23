use super::kind::NodeKind;
use super::NeopressNode;
use crate::schema::items::PriceFeatureItem;
use crate::schema::styles::StyleProperties;

#[test]
fn test_roundtrip_pro_blocks_serialization() {
    let node = NeopressNode {
        id: "price_card_1".to_string(),
        kind: NodeKind::PriceCard {
            title: "Enterprise Plan".to_string(),
            subtitle: Some("High scale installations".to_string()),
            currency: Some("$".to_string()),
            price: "99".to_string(),
            original_price: Some("149".to_string()),
            period: Some("/month".to_string()),
            badge: Some("Most Popular".to_string()),
            featured: true,
            features: vec![
                PriceFeatureItem {
                    text: "Unlimited Subdomains".to_string(),
                    included: true,
                    icon: Some("fa-solid fa-check".to_string()),
                },
                PriceFeatureItem {
                    text: "Dedicated Support".to_string(),
                    included: false,
                    icon: None,
                },
            ],
            button_text: "Upgrade Now".to_string(),
            button_url: "https://neopress.org/checkout".to_string(),
        },
        styles: StyleProperties {
            grid_template_columns: Some("repeat(3, 1fr)".to_string()),
            position: Some("relative".to_string()),
            tablet: Some(Box::new(StyleProperties {
                padding_top: Some("30px".to_string()),
                ..Default::default()
            })),
            hover: Some(Box::new(StyleProperties {
                background_color: Some("#2563eb".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        },
        children: Vec::new(),
    };

    let json = serde_json::to_string(&node).expect("Serialization failed");
    assert!(json.contains("\"type\":\"PriceCard\""));
    assert!(json.contains("\"title\":\"Enterprise Plan\""));
    assert!(json.contains("\"grid_template_columns\":\"repeat(3, 1fr)\""));
    assert!(json.contains("\"tablet\""));
    assert!(json.contains("\"hover\""));

    let deserialized: NeopressNode =
        serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(deserialized.id, "price_card_1");
    match &deserialized.kind {
        NodeKind::PriceCard {
            title,
            featured,
            features,
            ..
        } => {
            assert_eq!(title, "Enterprise Plan");
            assert!(featured);
            assert_eq!(features.len(), 2);
            assert!(features[0].included);
            assert!(!features[1].included);
        }
        _ => panic!("Expected PriceCard"),
    }
    assert_eq!(
        deserialized.styles.grid_template_columns.as_deref(),
        Some("repeat(3, 1fr)")
    );
    assert_eq!(
        deserialized
            .styles
            .tablet
            .as_ref()
            .and_then(|t| t.padding_top.as_deref()),
        Some("30px")
    );
    assert_eq!(
        deserialized
            .styles
            .hover
            .as_ref()
            .and_then(|h| h.background_color.as_deref()),
        Some("#2563eb")
    );
}
