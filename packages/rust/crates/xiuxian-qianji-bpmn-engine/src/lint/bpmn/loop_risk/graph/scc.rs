use super::{BpmnProcessSpec, outgoing_edge_indices};

pub(in crate::lint::bpmn::loop_risk) fn strongly_connected_components(
    process: &BpmnProcessSpec,
) -> Vec<Vec<usize>> {
    let mut tarjan = Tarjan::new(process);
    for node_index in 0..process.nodes.len() {
        if tarjan.indices[node_index].is_none() {
            tarjan.connect(node_index);
        }
    }
    tarjan.components
}

pub(in crate::lint::bpmn::loop_risk) struct Tarjan<'a> {
    process: &'a BpmnProcessSpec,
    next_index: usize,
    stack: Vec<usize>,
    on_stack: Vec<bool>,
    indices: Vec<Option<usize>>,
    lowlinks: Vec<usize>,
    components: Vec<Vec<usize>>,
}

impl<'a> Tarjan<'a> {
    fn new(process: &'a BpmnProcessSpec) -> Self {
        let node_count = process.nodes.len();
        Self {
            process,
            next_index: 0,
            stack: Vec::new(),
            on_stack: vec![false; node_count],
            indices: vec![None; node_count],
            lowlinks: vec![0; node_count],
            components: Vec::new(),
        }
    }

    fn connect(&mut self, node_index: usize) {
        self.indices[node_index] = Some(self.next_index);
        self.lowlinks[node_index] = self.next_index;
        self.next_index += 1;
        self.stack.push(node_index);
        self.on_stack[node_index] = true;

        if let Some(edge_indices) = outgoing_edge_indices(self.process, node_index) {
            for edge_index in edge_indices {
                let target_index = self.process.edges[*edge_index as usize].to as usize;
                if self.indices[target_index].is_none() {
                    self.connect(target_index);
                    self.lowlinks[node_index] =
                        self.lowlinks[node_index].min(self.lowlinks[target_index]);
                } else if self.on_stack[target_index] {
                    let target_order = self.indices[target_index].unwrap_or_default();
                    self.lowlinks[node_index] = self.lowlinks[node_index].min(target_order);
                }
            }
        }

        if self.lowlinks[node_index] == self.indices[node_index].unwrap_or_default() {
            let mut component = Vec::new();
            while let Some(member_index) = self.stack.pop() {
                self.on_stack[member_index] = false;
                component.push(member_index);
                if member_index == node_index {
                    break;
                }
            }
            component.sort_unstable();
            self.components.push(component);
        }
    }
}
