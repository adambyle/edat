use super::*;

pub struct WidgetData {
    pub name: String,
    pub description: String,
    pub order: Option<usize>,
    pub id: String,
}

pub fn widget_options_component(widgets: Vec<WidgetData>) -> Markup {
    html! {
        @for widget in widgets {
            .widget {
                @if let Some(order) = widget.order {
                    span style="opacity: 1" { "#" (order + 1) }
                } @else {
                    span {}
                }
                button #(widget.id) .selected[widget.order.is_some()] {
                    h3 { (PreEscaped(&widget.name)) }
                    p { (PreEscaped(&widget.description)) }
                }
            }
        }
    }
}

pub fn ordered_widget_data(selected: &[String]) -> Vec<WidgetData> {
    use WidgetData as W;

    let order = |id| selected.iter().position(|s| s == id);

    vec![
        W {
            name: "Recent additions".to_owned(),
            description: "Carousel of the latest sections".to_owned(),
            order: order(&"recent-widget"),
            id: "recent-widget".to_owned(),
        },
        W {
            name: "The library".to_owned(),
            description: "Quick access to the main journal’s four books".to_owned(),
            order: order(&"library-widget"),
            id: "library-widget".to_owned(),
        },
        W {
            name: "Last read".to_owned(),
            description: "Return to where you left off".to_owned(),
            order: order(&"last-widget"),
            id: "last-widget".to_owned(),
        },
        W {
            name: "Conversations".to_owned(),
            description: "See where readers have recently commented".to_owned(),
            order: order(&"conversations-widget"),
            id: "conversations-widget".to_owned(),
        },
        W {
            name: "Reading recommendation".to_owned(),
            description: "Based on what you have left to read".to_owned(),
            order: order(&"random-widget"),
            id: "random-widget".to_owned(),
        },
        W {
            name: "Extras".to_owned(),
            description: "Quick access to old journals, fiction, and more".to_owned(),
            order: order(&"extras-widget"),
            id: "extras-widget".to_owned(),
        },
        W {
            name: "Search bar".to_owned(),
            description: "Website search features".to_owned(),
            order: order(&"search-widget"),
            id: "search-widget".to_owned(),
        },
    ]
}
