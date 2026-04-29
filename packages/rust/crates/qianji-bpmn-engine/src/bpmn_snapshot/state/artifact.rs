use super::{
    ArtifactMetadataOwner, BpmnAssociationSnapshot, BpmnGroupSnapshot, BpmnSnapshotScanState,
    BpmnSourceFile, BpmnTextAnnotationSnapshot, BytesStart, Reader, Result, attribute_value,
};

impl BpmnSnapshotScanState {
    pub(super) fn capture_artifact_association(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(owner) = self.current_artifact_owner() else {
            return Ok(());
        };
        let association = BpmnAssociationSnapshot {
            association_id: attribute_value(source, reader, event, "id")?,
            source_ref: attribute_value(source, reader, event, "sourceRef")?,
            target_ref: attribute_value(source, reader, event, "targetRef")?,
            association_direction: attribute_value(source, reader, event, "associationDirection")?,
        };
        self.push_artifact_association(owner, association);
        Ok(())
    }

    pub(super) fn capture_artifact_group(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<()> {
        let Some(owner) = self.current_artifact_owner() else {
            return Ok(());
        };
        let group = BpmnGroupSnapshot {
            group_id: attribute_value(source, reader, event, "id")?,
            category_value_ref: attribute_value(source, reader, event, "categoryValueRef")?,
        };
        self.push_artifact_group(owner, group);
        Ok(())
    }

    pub(super) fn start_text_annotation(
        &mut self,
        source: &BpmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        is_empty: bool,
    ) -> Result<()> {
        let Some(owner) = self.current_artifact_owner() else {
            return Ok(());
        };
        let annotation = BpmnTextAnnotationSnapshot {
            annotation_id: attribute_value(source, reader, event, "id")?,
            text_format: attribute_value(source, reader, event, "textFormat")?,
            text: None,
        };
        if is_empty {
            self.push_text_annotation(owner, annotation);
            return Ok(());
        }
        self.current_text_annotation = Some((owner, annotation));
        Ok(())
    }

    pub(super) fn finish_text_annotation(&mut self) {
        let Some((owner, mut annotation)) = self.current_text_annotation.take() else {
            return;
        };
        annotation.text = annotation
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(ToString::to_string);
        self.push_text_annotation(owner, annotation);
    }

    pub(super) fn current_artifact_owner(&self) -> Option<ArtifactMetadataOwner> {
        if let Some(collaboration_index) = self.current_collaboration {
            return Some(ArtifactMetadataOwner::Collaboration(collaboration_index));
        }
        self.current_process.map(ArtifactMetadataOwner::Process)
    }

    pub(super) fn push_artifact_association(
        &mut self,
        owner: ArtifactMetadataOwner,
        association: BpmnAssociationSnapshot,
    ) {
        match owner {
            ArtifactMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.associations.push(association);
            }
            ArtifactMetadataOwner::Process(process_index) => {
                let Some(process) = self.processes.get_mut(process_index) else {
                    return;
                };
                process.association_count += 1;
                process.associations.push(association);
            }
        }
    }

    pub(super) fn push_artifact_group(
        &mut self,
        owner: ArtifactMetadataOwner,
        group: BpmnGroupSnapshot,
    ) {
        match owner {
            ArtifactMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.groups.push(group);
            }
            ArtifactMetadataOwner::Process(process_index) => {
                let Some(process) = self.processes.get_mut(process_index) else {
                    return;
                };
                process.group_count += 1;
                process.groups.push(group);
            }
        }
    }

    pub(super) fn push_text_annotation(
        &mut self,
        owner: ArtifactMetadataOwner,
        annotation: BpmnTextAnnotationSnapshot,
    ) {
        match owner {
            ArtifactMetadataOwner::Collaboration(collaboration_index) => {
                let Some(collaboration) = self.collaborations.get_mut(collaboration_index) else {
                    return;
                };
                collaboration.text_annotations.push(annotation);
            }
            ArtifactMetadataOwner::Process(process_index) => {
                let Some(process) = self.processes.get_mut(process_index) else {
                    return;
                };
                process.text_annotation_count += 1;
                process.text_annotations.push(annotation);
            }
        }
    }
}
