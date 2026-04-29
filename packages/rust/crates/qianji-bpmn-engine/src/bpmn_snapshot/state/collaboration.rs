use super::{
    BpmnChoreographyActivitySnapshot, BpmnCollaborationSnapshot,
    BpmnConversationAssociationSnapshot, BpmnConversationLinkSnapshot,
    BpmnConversationNodeSnapshot, BpmnCorrelationKeySnapshot, BpmnMessageFlowAssociationSnapshot,
    BpmnMessageFlowSnapshot, BpmnParticipantAssociationSnapshot,
    BpmnParticipantMultiplicitySnapshot, BpmnParticipantSnapshot, BpmnSnapshotScanState,
    BpmnSourceFile, BytesStart, CollaborationMetadataOwner, Reader, Result, attribute_value,
    boolean_attribute_value,
};

impl BpmnSnapshotScanState {
    pub(super) fn start_collaboration(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let collaboration = BpmnCollaborationSnapshot {
            collaboration_kind: tag.to_string(),
            collaboration_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            is_closed: boolean_attribute_value(source, reader, event, "isClosed")?,
            initiating_participant_ref: attribute_value(
                source,
                reader,
                event,
                "initiatingParticipantRef",
            )?,
            participants: Vec::new(),
            message_flows: Vec::new(),
            conversation_nodes: Vec::new(),
            conversation_associations: Vec::new(),
            participant_associations: Vec::new(),
            message_flow_associations: Vec::new(),
            correlation_keys: Vec::new(),
            choreography_refs: Vec::new(),
            choreography_activities: Vec::new(),
            conversation_links: Vec::new(),
            associations: Vec::new(),
            groups: Vec::new(),
            text_annotations: Vec::new(),
        };
        self.collaborations.push(collaboration);
        if let Some(root) = self.root.as_mut() {
            root.collaboration_count += 1;
        }
        if !is_empty {
            self.current_collaboration = self.collaborations.len().checked_sub(1);
        }
        Ok(())
    }

    pub(super) fn start_participant(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(collaboration_index) = self.current_collaboration else {
            return Ok(());
        };
        let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
            return Ok(());
        };
        collaboration.participants.push(BpmnParticipantSnapshot {
            participant_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            process_ref: attribute_value(source, reader, event, "processRef")?,
            interface_refs: Vec::new(),
            end_point_refs: Vec::new(),
            participant_multiplicity: None,
        });
        if !is_empty {
            let participant_index = collaboration.participants.len().saturating_sub(1);
            self.current_participant = Some((collaboration_index, participant_index));
        }
        Ok(())
    }

    pub(super) fn attach_participant_multiplicity(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let multiplicity = BpmnParticipantMultiplicitySnapshot {
            multiplicity_id: attribute_value(source, reader, event, "id")?,
            minimum: attribute_value(source, reader, event, "minimum")?,
            maximum: attribute_value(source, reader, event, "maximum")?,
        };
        let Some(participant) = self.current_participant_mut() else {
            return Ok(());
        };
        participant.participant_multiplicity = Some(multiplicity);
        Ok(())
    }

    pub(super) fn capture_message_flow(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration.message_flows.push(BpmnMessageFlowSnapshot {
            message_flow_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            source_ref: attribute_value(source, reader, event, "sourceRef")?,
            target_ref: attribute_value(source, reader, event, "targetRef")?,
            message_ref: attribute_value(source, reader, event, "messageRef")?,
        });
        Ok(())
    }

    pub(super) fn start_conversation_node(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(collaboration_index) = self.current_collaboration else {
            return Ok(());
        };
        let node = BpmnConversationNodeSnapshot {
            node_kind: tag.to_string(),
            node_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            called_collaboration_ref: attribute_value(
                source,
                reader,
                event,
                "calledCollaborationRef",
            )?,
            participant_refs: Vec::new(),
            message_flow_refs: Vec::new(),
            correlation_keys: Vec::new(),
            participant_associations: Vec::new(),
            child_nodes: Vec::new(),
        };
        let path = self.push_conversation_node(collaboration_index, node);
        if !is_empty {
            self.conversation_node_stack
                .push((collaboration_index, path));
        }
        Ok(())
    }

    pub(super) fn start_choreography_activity(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        tag: &str,
        is_empty: bool,
    ) -> Result<()> {
        let Some(collaboration_index) = self.current_collaboration else {
            return Ok(());
        };
        let activity = BpmnChoreographyActivitySnapshot {
            activity_kind: tag.to_string(),
            activity_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            initiating_participant_ref: attribute_value(
                source,
                reader,
                event,
                "initiatingParticipantRef",
            )?,
            loop_type: attribute_value(source, reader, event, "loopType")?,
            called_choreography_ref: attribute_value(
                source,
                reader,
                event,
                "calledChoreographyRef",
            )?,
            participant_refs: Vec::new(),
            message_flow_refs: Vec::new(),
            correlation_keys: Vec::new(),
            participant_associations: Vec::new(),
            child_activities: Vec::new(),
        };
        let path = self.push_choreography_activity(collaboration_index, activity);
        if !is_empty {
            self.choreography_activity_stack
                .push((collaboration_index, path));
        }
        Ok(())
    }

    pub(super) fn capture_conversation_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration
            .conversation_associations
            .push(BpmnConversationAssociationSnapshot {
                association_id: attribute_value(source, reader, event, "id")?,
                inner_conversation_node_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "innerConversationNodeRef",
                )?,
                outer_conversation_node_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "outerConversationNodeRef",
                )?,
            });
        Ok(())
    }

    pub(super) fn start_participant_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_collaboration_metadata_owner() else {
            return Ok(());
        };
        let association = BpmnParticipantAssociationSnapshot {
            association_id: attribute_value(source, reader, event, "id")?,
            inner_participant_ref: None,
            outer_participant_ref: None,
        };
        if is_empty {
            self.push_participant_association(owner, association);
            return Ok(());
        }
        self.current_participant_association = Some((owner, association));
        Ok(())
    }

    pub(super) fn capture_message_flow_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration
            .message_flow_associations
            .push(BpmnMessageFlowAssociationSnapshot {
                association_id: attribute_value(source, reader, event, "id")?,
                inner_message_flow_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "innerMessageFlowRef",
                )?,
                outer_message_flow_ref: attribute_value(
                    source,
                    reader,
                    event,
                    "outerMessageFlowRef",
                )?,
            });
        Ok(())
    }

    pub(super) fn start_conversation_correlation_key(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_collaboration_metadata_owner() else {
            return Ok(());
        };
        let key = BpmnCorrelationKeySnapshot {
            correlation_key_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            correlation_property_refs: Vec::new(),
        };
        if is_empty {
            self.push_conversation_correlation_key(owner, key);
            return Ok(());
        }
        self.current_conversation_correlation_key = Some((owner, key));
        Ok(())
    }

    pub(super) fn capture_conversation_link(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(collaboration) = self.current_collaboration_mut() else {
            return Ok(());
        };
        collaboration
            .conversation_links
            .push(BpmnConversationLinkSnapshot {
                link_id: attribute_value(source, reader, event, "id")?,
                name: attribute_value(source, reader, event, "name")?,
                source_ref: attribute_value(source, reader, event, "sourceRef")?,
                target_ref: attribute_value(source, reader, event, "targetRef")?,
            });
        Ok(())
    }

    pub(super) fn finish_conversation_correlation_key(&mut self) {
        let Some((owner, key)) = self.current_conversation_correlation_key.take() else {
            return;
        };
        self.push_conversation_correlation_key(owner, key);
    }

    pub(super) fn push_conversation_correlation_key(
        &mut self,
        owner: CollaborationMetadataOwner,
        key: BpmnCorrelationKeySnapshot,
    ) {
        match owner {
            CollaborationMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.correlation_keys.push(key);
            }
            CollaborationMetadataOwner::ConversationNode(collaboration_index, path) => {
                let Some(node) = self.conversation_node_mut(collaboration_index, &path) else {
                    return;
                };
                node.correlation_keys.push(key);
            }
            CollaborationMetadataOwner::ChoreographyActivity(collaboration_index, path) => {
                let Some(activity) = self.choreography_activity_mut(collaboration_index, &path)
                else {
                    return;
                };
                activity.correlation_keys.push(key);
            }
        }
    }

    pub(super) fn finish_participant_association(&mut self) {
        let Some((owner, association)) = self.current_participant_association.take() else {
            return;
        };
        self.push_participant_association(owner, association);
    }

    pub(super) fn push_participant_association(
        &mut self,
        owner: CollaborationMetadataOwner,
        association: BpmnParticipantAssociationSnapshot,
    ) {
        match owner {
            CollaborationMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.participant_associations.push(association);
            }
            CollaborationMetadataOwner::ConversationNode(collaboration_index, path) => {
                let Some(node) = self.conversation_node_mut(collaboration_index, &path) else {
                    return;
                };
                node.participant_associations.push(association);
            }
            CollaborationMetadataOwner::ChoreographyActivity(collaboration_index, path) => {
                let Some(activity) = self.choreography_activity_mut(collaboration_index, &path)
                else {
                    return;
                };
                activity.participant_associations.push(association);
            }
        }
    }

    pub(super) fn current_collaboration_mut(&mut self) -> Option<&mut BpmnCollaborationSnapshot> {
        self.current_collaboration
            .and_then(|index| self.collaborations.get_mut(index))
    }

    pub(super) fn current_participant_mut(&mut self) -> Option<&mut BpmnParticipantSnapshot> {
        let (collaboration_index, participant_index) = self.current_participant?;
        self.collaborations
            .get_mut(collaboration_index)?
            .participants
            .get_mut(participant_index)
    }

    pub(super) fn current_collaboration_metadata_owner(
        &self,
    ) -> Option<CollaborationMetadataOwner> {
        if let Some((collaboration_index, path)) = self.choreography_activity_stack.last() {
            return Some(CollaborationMetadataOwner::ChoreographyActivity(
                *collaboration_index,
                path.clone(),
            ));
        }
        if let Some((collaboration_index, path)) = self.conversation_node_stack.last() {
            return Some(CollaborationMetadataOwner::ConversationNode(
                *collaboration_index,
                path.clone(),
            ));
        }
        self.current_collaboration
            .map(CollaborationMetadataOwner::Collaboration)
    }

    pub(super) fn push_conversation_node(
        &mut self,
        collaboration_index: usize,
        node: BpmnConversationNodeSnapshot,
    ) -> Vec<usize> {
        let Some((_, parent_path)) = self.conversation_node_stack.last().cloned() else {
            let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                return Vec::new();
            };
            collaboration.conversation_nodes.push(node);
            return vec![collaboration.conversation_nodes.len().saturating_sub(1)];
        };
        let Some(parent) = self.conversation_node_mut(collaboration_index, &parent_path) else {
            return parent_path;
        };
        parent.child_nodes.push(node);
        let mut path = parent_path;
        path.push(parent.child_nodes.len().saturating_sub(1));
        path
    }

    pub(super) fn push_choreography_activity(
        &mut self,
        collaboration_index: usize,
        activity: BpmnChoreographyActivitySnapshot,
    ) -> Vec<usize> {
        let Some((_, parent_path)) = self.choreography_activity_stack.last().cloned() else {
            let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                return Vec::new();
            };
            collaboration.choreography_activities.push(activity);
            return vec![
                collaboration
                    .choreography_activities
                    .len()
                    .saturating_sub(1),
            ];
        };
        let Some(parent) = self.choreography_activity_mut(collaboration_index, &parent_path) else {
            return parent_path;
        };
        parent.child_activities.push(activity);
        let mut path = parent_path;
        path.push(parent.child_activities.len().saturating_sub(1));
        path
    }

    pub(super) fn current_conversation_node_mut(
        &mut self,
    ) -> Option<&mut BpmnConversationNodeSnapshot> {
        let (collaboration_index, path) = self.conversation_node_stack.last().cloned()?;
        self.conversation_node_mut(collaboration_index, &path)
    }

    pub(super) fn current_choreography_activity_mut(
        &mut self,
    ) -> Option<&mut BpmnChoreographyActivitySnapshot> {
        let (collaboration_index, path) = self.choreography_activity_stack.last().cloned()?;
        self.choreography_activity_mut(collaboration_index, &path)
    }

    pub(super) fn conversation_node_mut(
        &mut self,
        collaboration_index: usize,
        path: &[usize],
    ) -> Option<&mut BpmnConversationNodeSnapshot> {
        let (first, rest) = path.split_first()?;
        let mut node = self
            .collaborations
            .get_mut(collaboration_index)?
            .conversation_nodes
            .get_mut(*first)?;
        for index in rest {
            node = node.child_nodes.get_mut(*index)?;
        }
        Some(node)
    }

    pub(super) fn choreography_activity_mut(
        &mut self,
        collaboration_index: usize,
        path: &[usize],
    ) -> Option<&mut BpmnChoreographyActivitySnapshot> {
        let (first, rest) = path.split_first()?;
        let mut activity = self
            .collaborations
            .get_mut(collaboration_index)?
            .choreography_activities
            .get_mut(*first)?;
        for index in rest {
            activity = activity.child_activities.get_mut(*index)?;
        }
        Some(activity)
    }
}
