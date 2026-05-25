# BPMN Official Source Map

This module anchors the local BPMN alignment notes to the official OMG
`BPMN 2.0.2` inventory instead of relying on recollection or secondary
summaries.

## Official Source Bundle

- [About BPMN 2.0.2](https://www.omg.org/spec/BPMN/2.0.2/About-BPMN)
- [Normative BPMN 2.0.2 PDF](https://www.omg.org/spec/BPMN/2.0.2/PDF)
- [BPMN20.xsd](https://www.omg.org/spec/BPMN/20100501/BPMN20.xsd)
- [Semantic.xsd](https://www.omg.org/spec/BPMN/20100501/Semantic.xsd)
- [BPMNDI.xsd](https://www.omg.org/spec/BPMN/20100501/BPMNDI.xsd)
- [DC.xsd](https://www.omg.org/spec/BPMN/20100501/DC.xsd)
- [DI.xsd](https://www.omg.org/spec/BPMN/20100501/DI.xsd)

## Module Map

| Local module                                                                                   | Primary OMG clauses                                                                                                                                                                        | Official assets                                                       | Local reading                                                                                    |
| ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| [README.md](README.md)                                                                         | `2.2 Process Modeling Conformance`; `2.3 Process Execution Conformance`; `7 Overview`; `8 BPMN Core Structure`; `9 Collaboration`; `10 Process`                                            | About page; normative PDF; `BPMN20.xsd`; `Semantic.xsd`; `BPMNDI.xsd` | family index and bounded package surface                                                         |
| [collaboration-lanes-and-data.md](collaboration-lanes-and-data.md)                             | `9 Collaboration`; `9.3 Pool and Participant`; `9.3.2 Lanes`; `9.4 Message Flow`; `9.5 Choreography`; `10.4 Items and Data`; `10.8 Lanes`; partner, endpoint, and artifact schema surfaces | normative PDF; `BPMN20.xsd`; `Semantic.xsd`; `BPMNDI.xsd`             | snapshot- and lint-owned collaboration, partner, choreography, artifact, lane, and data families |
| [tasks-and-host-dispatch.md](tasks-and-host-dispatch.md)                                       | `10.3 Activities`; `10.3.3 Tasks`; `10.3.4 Human Interactions`; `10.4 Items and Data`                                                                                                      | normative PDF; `BPMN20.xsd`; `Semantic.xsd`                           | accepted task families and host seam                                                             |
| [human-interaction-milestone-plan.md](human-interaction-milestone-plan.md)                     | `10.3.3 Tasks`; `10.3.4 Human Interactions`; standard task/resource-role schema surfaces                                                                                                   | normative PDF; `Semantic.xsd`                                         | source-backed human interaction milestone matrix                                                 |
| [host-request-abi-ledger.md](host-request-abi-ledger.md)                                       | `10.3.3 Tasks`; `10.3.4 Human Interactions`; standard task/resource-role schema surfaces                                                                                                   | normative PDF; `Semantic.xsd`                                         | user/manual host request field ledger                                                            |
| [events-and-boundaries.md](events-and-boundaries.md)                                           | `8.4.5 Events`; `10.5 Events`; `10.7 Compensation`; `13.5 Events`                                                                                                                          | normative PDF; `BPMN20.xsd`; `Semantic.xsd`                           | catch, boundary, cancel, and bounded event families                                              |
| [gateways-and-concurrency.md](gateways-and-concurrency.md)                                     | `8.4.9 Gateways`; `10.6 Gateways`; `13.4 Gateways`; `2.3.1 Execution Semantics`                                                                                                            | normative PDF; `BPMN20.xsd`; `Semantic.xsd`                           | split, join, and event-based gateway runtime                                                     |
| [loops-and-multi-instance.md](loops-and-multi-instance.md)                                     | `10.3.8 Loop Characteristics`; `10.4 Items and Data`                                                                                                                                       | normative PDF; `BPMN20.xsd`; `Semantic.xsd`                           | bounded standard-loop and multi-instance behavior                                                |
| [subprocesses-transactions-and-compensation.md](subprocesses-transactions-and-compensation.md) | `10.3.5 Sub-Processes`; `10.3.6 Call Activity`; `10.7 Compensation`; `13.3.4 Sub-Process/Call Activity`; `13.5.5 Compensation`                                                             | normative PDF; `BPMN20.xsd`; `Semantic.xsd`                           | bounded nested scopes and transaction-owned compensation                                         |

## Reading Rule

These mappings are intentionally many-to-one. Each local module groups the OMG
clauses that the crate currently needs to keep one bounded parser, runtime, or
lint family honest. They are not a claim that one local page implements every
semantic rule in those clauses.
