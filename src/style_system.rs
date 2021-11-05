use crate::{Context, Display, Entity, Rule, Tree, TreeExt, Visibility, style::{Property, Selector, SelectorRelation}};

// use crate::{BoundingBox, Display, Entity, Overflow, PropGet, PropSet, Property, SelectorRelation, Rule, Selector, Cx, Tree, TreeExt, Visibility};


pub fn apply_z_ordering(cx: &mut Context, tree: &Tree) {
    for entity in tree.into_iter() {
        if entity == Entity::root() {
            continue;
        }

        let parent = tree.get_parent(entity).unwrap();

        if let Some(z_order) = cx.style.borrow_mut().z_order.get(entity) {
            cx.cache.set_z_index(entity, *z_order);
        } else {
            let parent_z_order = cx.cache.get_z_index(parent);
            cx.cache.set_z_index(entity, parent_z_order);
        }
    }
}
/* TODO
pub fn apply_clipping(cx: &mut Context, tree: &Tree) {
    //println!("Apply Clipping");
    for entity in tree.into_iter() {
        if entity == Entity::root() {
            continue;
        }

        let parent = tree.get_parent(entity).unwrap();

        let mut parent_clip_region = cx.cache.get_clip_region(parent);
        let parent_border_width = cx.style.borrow_mut().border_width.get(parent).cloned().unwrap_or_default().value_or(0.0, 0.0);

        //println!("Parent border width: {}", parent_border_width);
        parent_clip_region.x += parent_border_width / 2.0;
        parent_clip_region.y += parent_border_width / 2.0;
        parent_clip_region.w -= parent_border_width;
        parent_clip_region.h -= parent_border_width;



        let root_clip_region = cx.cache.get_clip_region(Entity::root());

        if entity.get_overflow(cx) == Overflow::Hidden {
            if let Some(clip_widget) = cx.style.borrow_mut().clip_widget.get(entity).cloned() {
                let clip_widget_border_width = cx.style.borrow_mut().border_width.get(clip_widget).cloned().unwrap_or_default().value_or(0.0, 0.0);
                let clip_x = cx.cache.get_posx(clip_widget) + clip_widget_border_width;
                let clip_y = cx.cache.get_posy(clip_widget) + clip_widget_border_width;
                let clip_w = cx.cache.get_width(clip_widget) - 2.0 * clip_widget_border_width;
                let clip_h = cx.cache.get_height(clip_widget) - 2.0 * clip_widget_border_width;

                let mut intersection = BoundingBox::default();
                intersection.x = clip_x.max(parent_clip_region.x);
                intersection.y = clip_y.max(parent_clip_region.y);

                intersection.w = if clip_x + clip_w < parent_clip_region.x + parent_clip_region.w {
                    clip_x + clip_w - intersection.x
                } else {
                    parent_clip_region.x + parent_clip_region.w - intersection.x
                };

                intersection.h = if clip_y + clip_h < parent_clip_region.y + parent_clip_region.h {
                    clip_y + clip_h - intersection.y
                } else {
                    parent_clip_region.y + parent_clip_region.h - intersection.y
                };

                cx.cache.set_clip_region(entity, intersection);
            } else {
                cx.cache.set_clip_region(entity, parent_clip_region);
            }
        } else {
            cx.cache.set_clip_region(entity, root_clip_region);
        }

        //let clip_region = cx.cache.get_clip_region(entity);
        //println!("Entity: {}  Clip Region: {:?}", entity, clip_region);
    }
}
*/

pub fn apply_visibility(cx: &mut Context, tree: &Tree) {
    let mut draw_tree: Vec<Entity> = tree.into_iter().collect();
    draw_tree.sort_by_cached_key(|entity| cx.cache.get_z_index(*entity));

    for entity in draw_tree.into_iter() {

        if entity == Entity::root() {
            continue;
        }

        let parent= entity.parent(tree).unwrap();

        if cx.cache.get_visibility(parent) == Visibility::Invisible {
            cx.cache.set_visibility(entity, Visibility::Invisible);
        } else {
            if let Some(visibility) = cx.style.borrow_mut().visibility.get(entity) {
                cx.cache.set_visibility(entity, *visibility);
            } else {
                cx.cache.set_visibility(entity, Visibility::Visible);
            }
        }

        if cx.cache.get_display(parent) == Display::None {
            cx.cache.set_display(entity, Display::None);
        } else {
            if let Some(display) = cx.style.borrow_mut().display.get(entity) {
                cx.cache.set_display(entity, *display);
            } else {
                cx.cache.set_display(entity, Display::Flex);
            }
        }

        let parent_opacity = cx.cache.get_opacity(parent);

        let opacity = cx.style.borrow_mut().opacity.get(entity).cloned().unwrap_or_default();

        cx.cache.set_opacity(entity, opacity.0 * parent_opacity);
    }
}

// Returns true if the widget matches the selector
fn check_match(cx: &Context, entity: Entity, selector: &Selector) -> bool {

    // Universal selector always matches
    if selector.asterisk {
        if let Some(pseudo_classes) = cx.style.borrow().pseudo_classes.get(entity) {
            if !pseudo_classes.is_empty() && !pseudo_classes.intersects(*pseudo_classes)
            {
                return false;
            } else {
                return true;
            }            
        } else {
            return true;
        }
    }

    // Check for ID match TODO
    // if selector.id.is_some() && selector.id != entity_selector.id {
    //     return false;
    // }


    // Check for element name match
    if let Some(selector_element) = &selector.element {
        if let Some(element) = cx.style.borrow().elements.get(entity) {
            if selector_element != element {
                return false;
            }
        } else {
            return false;
        }
    }

    // Check for classes match
    if let Some(classes) = cx.style.borrow().classes.get(entity) {
        if !selector.classes.is_subset(classes) {
            return false;
        }        
    } else if !selector.classes.is_empty() {
        return false;
    }

    // Check for pseudo-class match
    if let Some(pseudo_classes) = cx.style.borrow().pseudo_classes.get(entity) {
        if !selector.pseudo_classes.is_empty() && !selector.pseudo_classes.intersects(*pseudo_classes)
        {
            return false;
        }
    }

    return true;
}

pub fn apply_styles(cx: &mut Context, tree: &Tree) {
    //println!("RESTYLE");
    // Loop through all entities
    for entity in tree.into_iter() {
        // Skip the root
        if entity == Entity::root() {
            continue;
        }

        // Create a list of style.borrow_mut() rules that match this entity
        let mut matched_rules: Vec<Rule> = Vec::new();

        // Loop through all of the style.borrow_mut() rules
        'rule_loop: for rule in cx.style.borrow().rules.iter() {
            let mut relation_entity = entity;
            // Loop through selectors (Should be from right to left)
            // All the selectors need to match for the rule to apply
            'selector_loop: for rule_selector in rule.selectors.iter().rev() {
                // Get the relation of the selector
                match rule_selector.relation {
                    SelectorRelation::None => {
                        if !check_match(cx, entity, rule_selector) {
                            continue 'rule_loop;
                        }
                    }

                    SelectorRelation::Parent => {
                        // Get the parent
                        // Contrust the selector for the parent
                        // Check if the parent selector matches the rule_seletor
                        if let Some(parent) = relation_entity.parent(tree) {
                            if !check_match(cx, parent, rule_selector) {
                                continue 'rule_loop;
                            }

                            relation_entity = parent;
                        } else {
                            continue 'rule_loop;
                        }
                    }

                    SelectorRelation::Ancestor => {
                        // Walk up the tree
                        // Check if each entity matches the selector
                        // If any of them match, move on to the next selector
                        // If none of them do, move on to the next rule
                        for ancestor in relation_entity.parent_iter(tree) {
                            if ancestor == relation_entity {
                                continue;
                            }

                            if check_match(cx, ancestor, rule_selector) {
                                relation_entity = ancestor;

                                continue 'selector_loop;
                            }
                        }

                        continue 'rule_loop;
                    }
                }
            }

            // If all the selectors match then add the rule to the matched rules list
            matched_rules.push(rule.id);
        }

        //println!("Entity: {}, Matched Rules: {:?}", entity, &matched_rules);

        // if matched_rules.len() == 0 {
        //     continue;
        // }

        let mut _should_relayout = false;
        let mut _should_redraw = false;

        // Display
        if cx.style.borrow_mut().display.link(entity, &matched_rules) {
            //println!("1");
            _should_relayout = true;
            _should_redraw = true;
        }
        if cx.style.borrow_mut().visibility.link(entity, &matched_rules) {
            //println!("2");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().z_order.link(entity, &matched_rules) {
            //println!("3");
            _should_relayout = true;
            _should_redraw = true;
        }

        // Currently doesn't do anything - TODO
        cx.style.borrow_mut().overflow.link(entity, &matched_rules);

        // Opacity
        if cx.style.borrow_mut().opacity.link(entity, &matched_rules) {
            //println!("4");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().left.link(entity, &matched_rules) {
            //println!("6");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().right.link(entity, &matched_rules) {
            //println!("7");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().top.link(entity, &matched_rules) {
            //println!("8");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().bottom.link(entity, &matched_rules) {
            //println!("9");
            _should_relayout = true;
            _should_redraw = true;
        }

        // Size
        if cx.style.borrow_mut().width.link(entity, &matched_rules) {
            //println!("10");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().height.link(entity, &matched_rules) {
            //println!("11");
            _should_relayout = true;
            _should_redraw = true;
        }

        // Size Constraints
        if cx.style.borrow_mut().max_width.link(entity, &matched_rules) {
            //println!("12");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().min_width.link(entity, &matched_rules) {
            //println!("13");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().max_height.link(entity, &matched_rules) {
            //println!("14");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().min_height.link(entity, &matched_rules) {
            //println!("15");
            _should_relayout = true;
            _should_redraw = true;
        }

        // Border
        if cx.style.borrow_mut().border_width.link(entity, &matched_rules) {
            //println!("24");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().border_color.link(entity, &matched_rules) {
            //println!("25");
            _should_redraw = true;
        }

        if cx.style.borrow_mut().border_shape_top_left.link(entity, &matched_rules) {
            _should_redraw = true;
        }

        if cx.style.borrow_mut().border_shape_top_right.link(entity, &matched_rules) {
            _should_redraw = true;
        }

        if cx.style.borrow_mut().border_shape_bottom_left.link(entity, &matched_rules) {
            _should_redraw = true;
        }

        if cx.style.borrow_mut().border_shape_bottom_right.link(entity, &matched_rules) {
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .border_radius_top_left
            .link(entity, &matched_rules)
        {
            //println!("26");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .border_radius_top_right
            .link(entity, &matched_rules)
        {
            //println!("27");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .border_radius_bottom_left
            .link(entity, &matched_rules)
        {
            //println!("28");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .border_radius_bottom_right
            .link(entity, &matched_rules)
        {
            //println!("29");
            _should_redraw = true;
        }

        if cx.style.borrow_mut().layout_type.link(entity, &matched_rules) {
            //println!("30");
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .position_type
            .link(entity, &matched_rules)
        {
            //println!("30");
            _should_relayout = true;
            _should_redraw = true;
        }

        // Background
        if cx
            .style.borrow_mut()
            .background_color
            .link(entity, &matched_rules)
        {
            //println!("41");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .background_image
            .link(entity, &matched_rules)
        {
            //println!("42");
            _should_redraw = true;
        }

        // Font
        if cx.style.borrow_mut().font_color.link(entity, &matched_rules) {
            //println!("43");
            _should_redraw = true;
        }

        if cx.style.borrow_mut().font_size.link(entity, &matched_rules) {
            //println!("44");
            _should_redraw = true;
        }

        if cx.style.borrow_mut().font.link(entity, &matched_rules) {
            //println!("44");
            _should_redraw = true;
        }

        // Outer Shadow
        if cx
            .style.borrow_mut()
            .outer_shadow_h_offset
            .link(entity, &matched_rules)
        {
            //println!("45");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .outer_shadow_v_offset
            .link(entity, &matched_rules)
        {
            //println!("46");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .outer_shadow_blur
            .link(entity, &matched_rules)
        {
            //println!("47");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .outer_shadow_color
            .link(entity, &matched_rules)
        {
            //println!("48");
            _should_redraw = true;
        }

        // Inner Shadow
        if cx
            .style.borrow_mut()
            .inner_shadow_h_offset
            .link(entity, &matched_rules)
        {
            //println!("45");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .inner_shadow_v_offset
            .link(entity, &matched_rules)
        {
            //println!("46");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .inner_shadow_blur
            .link(entity, &matched_rules)
        {
            //println!("47");
            _should_redraw = true;
        }

        if cx
            .style.borrow_mut()
            .inner_shadow_color
            .link(entity, &matched_rules)
        {
            //println!("48");
            _should_redraw = true;
        }

        if cx.style.borrow_mut().child_left.link(entity, &matched_rules) {
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().child_right.link(entity, &matched_rules) {
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().child_top.link(entity, &matched_rules) {
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().child_bottom.link(entity, &matched_rules) {
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().row_between.link(entity, &matched_rules) {
            _should_relayout = true;
            _should_redraw = true;
        }

        if cx.style.borrow_mut().col_between.link(entity, &matched_rules) {
            _should_relayout = true;
            _should_redraw = true;
        }


        if cx.style.borrow_mut().cursor.link(entity, &matched_rules) {
            //_should_relayout = true;
            //_should_redraw = true;
        }

        // for rule_id in matched_rules.iter() {
        //     // TODO - remove cloned
        //     if let Some(rule_index) = cx.style.borrow_mut().rules.iter().position(|rule| rule.id == *rule_id) {
        //         if let Some(rule) = cx.style.borrow_mut().rules.get(rule_index).cloned() {
        //             for property in rule.properties.iter() {
        //                 match property {
        //                     Property::Unknown(ident, prop) => {
        //                         if let Some(mut event_handler) = cx.event_handlers.remove(&entity) {
        //                             event_handler.on_style.borrow_mut()(cx, entity, (ident.clone(), prop.clone()));
    
        //                             cx.event_handlers.insert(entity, event_handler);
        //                         }
        //                     }
    
        //                     _=> {}
        //                 }
        //             }
        //         }
        //     }
        // }

        // if _should_relayout {
        //     Entity::root().relayout(cx);
        //     //cx.needs_relayout = true;
        // }

        // if _should_redraw {
        //     Entity::root().redraw(cx);
        //     //cx.needs_redraw = true;
        // }
    }

}