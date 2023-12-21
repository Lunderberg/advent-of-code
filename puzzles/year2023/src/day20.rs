use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use aoc_utils::prelude::*;
use bit_set::BitSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PulseKind {
    High,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pulse {
    kind: PulseKind,
    sender: usize,
    receiver: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleKind {
    Button,
    Broadcaster,
    FlipFlop,
    Conjunction,
    Output,
}

#[derive(Debug, Clone)]
struct Module {
    name: String,
    kind: ModuleKind,
    outputs_to: Vec<usize>,
}

#[derive(Debug)]
pub struct System {
    modules: Vec<Module>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ModuleState {
    Button,
    Broadcaster,

    /// Stores whether the FlipFlop is currently on.
    FlipFlop(bool),

    /// Stores the set of inputs whose most recent signal to the
    /// module was a low pulse, initialized as the set of all modules
    /// that are connected to the Conjunction module.  This storage
    /// avoids needing to track which module is connected to which of
    /// the Conjunction's inputs.
    Conjunction(BitSet),

    Output,
}

#[derive(Debug, Clone)]
struct SystemState<'a> {
    system: &'a System,
    module_states: Vec<ModuleState>,
}

// Only compare the module_states, not the sytem
impl<'a> std::hash::Hash for SystemState<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.module_states.hash(state);
    }
}
impl<'a> std::cmp::PartialEq for SystemState<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.module_states == other.module_states
    }
}
impl<'a> std::cmp::Eq for SystemState<'a> {}

impl Display for PulseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PulseKind::High => "High",
            PulseKind::Low => "Low",
        };
        write!(f, "{s}")
    }
}

// https://rust-lang.github.io/rfcs/3498-lifetime-capture-rules-2024.html#the-captures-trick
trait Captures<U> {}
impl<T: ?Sized, U> Captures<U> for T {}

impl System {
    fn initial_state(&self) -> SystemState<'_> {
        let module_states = self
            .modules
            .iter()
            .enumerate()
            .map(|(i, module)| match module.kind {
                ModuleKind::Button => ModuleState::Button,
                ModuleKind::Broadcaster => ModuleState::Broadcaster,
                ModuleKind::FlipFlop => ModuleState::FlipFlop(false),
                ModuleKind::Conjunction => {
                    let bit_set = self
                        .modules
                        .iter()
                        .enumerate()
                        .filter(|(_, module)| {
                            module.outputs_to.iter().any(|&out| out == i)
                        })
                        .map(|(j, _)| j)
                        .collect();
                    ModuleState::Conjunction(bit_set)
                }
                ModuleKind::Output => ModuleState::Output,
            })
            .collect();
        SystemState {
            system: self,
            module_states,
        }
    }

    fn find_kind(&self, kind: ModuleKind) -> Option<usize> {
        self.modules
            .iter()
            .enumerate()
            .find(|(_, module)| module.kind == kind)
            .map(|(i, _)| i)
    }

    fn make_button_press(&self) -> Pulse {
        let button =
            self.find_kind(ModuleKind::Button).expect("No button found");

        let broadcaster = self
            .find_kind(ModuleKind::Broadcaster)
            .expect("No broadcaster found");

        Pulse {
            kind: PulseKind::Low,
            sender: button,
            receiver: broadcaster,
        }
    }

    fn get_name(&self, index: usize) -> Option<&str> {
        self.modules.get(index).map(|module| module.name.as_str())
    }

    fn find_indirect_inputs(&self) -> HashMap<usize, BitSet<usize>> {
        let mut inputs_from: HashMap<usize, BitSet<usize>> = self
            .modules
            .iter()
            .enumerate()
            .flat_map(|(from, module)| {
                module.outputs_to.iter().cloned().map(move |to| (to, from))
            })
            .into_grouping_map()
            .collect();

        loop {
            let next: HashMap<usize, BitSet<usize>> = inputs_from
                .iter()
                .map(|(to, froms)| {
                    let froms = froms
                        .iter()
                        .flat_map(|from| {
                            inputs_from.get(&from).into_iter().flatten()
                        })
                        .collect();
                    (*to, froms)
                })
                .collect();

            let old_size = inputs_from
                .iter()
                .map(|(_, froms)| froms.len())
                .sum::<usize>();
            let new_size =
                next.iter().map(|(_, froms)| froms.len()).sum::<usize>();
            if old_size == new_size {
                break;
            } else {
                inputs_from = next;
            }
        }

        inputs_from
    }

    fn get_pruned_tree(&self, index: usize) -> Self {
        let required = self.find_indirect_inputs().remove(&index).unwrap();
        let modules = self
            .modules
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, mut module)| {
                if index == i || required.contains(i) {
                    module
                } else {
                    module.kind = ModuleKind::Output;
                    module.outputs_to = Vec::new();
                    module
                }
            })
            .collect();

        Self { modules }
    }

    fn find_cycle(&self, index: usize) -> (usize, usize) {
        let pruned = self.get_pruned_tree(index);
        let mut state = pruned.initial_state();
        let button = pruned.make_button_press();

        let mut seen: HashMap<SystemState, usize> = HashMap::new();
        seen.insert(state.clone(), seen.len());
        loop {
            state.process_single_pulse(button).for_each(|_| {});
            if let Some(prev) = seen.get(&state) {
                seen.iter()
                    .map(|(state, i)| (&state.module_states[index], i))
                    .sorted_by_key(|(_, i)| *i)
                    .filter(|(module_state, _)| match module_state {
                        ModuleState::Conjunction(bit_set) => {
                            !bit_set.is_empty()
                        }
                        _ => true,
                    })
                    .for_each(|(module_state, i)| {
                        println!("\t{i}: {module_state:?}")
                    });

                return (*prev, seen.len());
            } else {
                seen.insert(state.clone(), seen.len());
            }
        }
    }
}
impl<'sys> SystemState<'sys> {
    fn process_button_presses<'state>(
        &'state mut self,
        num_presses: usize,
    ) -> impl Iterator<Item = Pulse> + 'state + Captures<&'sys ()> {
        let pulse = self.system.make_button_press();

        (0..num_presses).flat_map(move |_| {
            self.process_single_pulse(pulse).collect_vec().into_iter()
        })
    }

    // fn repeatedly_press_button<'state>(
    //     &'state mut self,
    // ) -> impl Iterator<Item = Pulse> + 'state + Captures<&'sys ()> {
    //     let button = self
    //         .system
    //         .find_kind(ModuleKind::Button)
    //         .expect("No button found");
    //     let broadcaster = self
    //         .system
    //         .find_kind(ModuleKind::Broadcaster)
    //         .expect("No broadcaster found");
    //     let pulse = Pulse {
    //         kind: PulseKind::Low,
    //         sender: button,
    //         receiver: broadcaster,
    //     };

    //     std::iter::repeat(()).flat_map(move |_| {
    //         self.process_single_pulse(pulse).collect_vec().into_iter()
    //     })
    // }

    /// Process a single pulse, producing the pulses that are
    /// generated by the pulse's recipients, including all indirect
    /// results..
    fn process_single_pulse<'state>(
        &'state mut self,
        pulse: Pulse,
    ) -> impl Iterator<Item = Pulse> + 'state + Captures<&'sys ()> {
        let mut to_process: VecDeque<_> = vec![pulse].into();

        std::iter::from_fn(move || {
            let pulse = to_process.pop_front()?;

            self.process_single_pulse_impl(pulse)
                .for_each(|new_pulse| to_process.push_back(new_pulse));

            Some(pulse)
        })
    }

    /// Process a single pulse, producing the pulses that are
    /// immediately generated by the pulse's recipients.
    fn process_single_pulse_impl<'state>(
        &'state mut self,
        input_pulse: Pulse,
    ) -> impl Iterator<Item = Pulse> + 'state + Captures<&'sys ()> {
        let state = &mut self.module_states[input_pulse.receiver];
        let output_kind = match (state, input_pulse.kind) {
            (ModuleState::Broadcaster, kind) => Some(kind),
            (ModuleState::Output, _) => None,
            (ModuleState::FlipFlop(value), PulseKind::Low) => {
                *value = !*value;
                Some(match value {
                    true => PulseKind::High,
                    false => PulseKind::Low,
                })
            }
            (ModuleState::Conjunction(bit_set), kind) => {
                // println!("Conjunction with state {bit_set:?} received {kind}");
                match kind {
                    PulseKind::High => {
                        bit_set.remove(input_pulse.sender);
                    }
                    PulseKind::Low => {
                        bit_set.insert(input_pulse.sender);
                    }
                }
                // println!("\tConjunction now has state {bit_set:?}");

                Some(if bit_set.is_empty() {
                    PulseKind::Low
                } else {
                    PulseKind::High
                })
            }

            _ => None,
        };

        // println!(
        //     "Node {} received {}",
        //     self.system.modules[input_pulse.receiver].name, input_pulse.kind
        // );

        output_kind.into_iter().flat_map(move |kind| {
            self.system.modules[input_pulse.receiver]
                .outputs_to
                .iter()
                .copied()
                .map(move |receiver| Pulse {
                    kind,
                    sender: input_pulse.receiver,
                    receiver,
                })
            // .inspect(|output_pulse| {
            //     println!(
            //         "\tSending {} to {}",
            //         output_pulse.kind,
            //         self.system.modules[output_pulse.receiver].name
            //     );
            // })
        })
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    //const EXAMPLE_NUM: u8 = 0;
    const EXAMPLE_NUM: u8 = 2;

    type ParsedInput = System;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let module_specs: Vec<_> = lines
            .map(|line| -> Result<_, Error> {
                let (name, sent_to) = line
                    .split(" -> ")
                    .collect_tuple()
                    .ok_or(Error::WrongIteratorSize)?;

                let outputs_to: Vec<_> = sent_to.split(", ").collect();

                let kind = if name.starts_with("%") {
                    Ok(ModuleKind::FlipFlop)
                } else if name.starts_with("&") {
                    Ok(ModuleKind::Conjunction)
                } else if name == "broadcaster" {
                    Ok(ModuleKind::Broadcaster)
                } else {
                    Err(Error::InvalidString(line.to_string()))
                }?;

                let name = name.trim_start_matches('%').trim_start_matches('&');

                Ok((name, kind, outputs_to))
            })
            .collect::<Result<_, _>>()?;

        let output_names: Vec<_> = std::iter::empty()
            .chain(module_specs.iter().map(|(name, _, _)| name))
            .chain(
                module_specs
                    .iter()
                    .flat_map(|(_, _, outputs_to)| outputs_to),
            )
            .unique()
            .skip(module_specs.len())
            .collect();

        let name_to_index: HashMap<_, _> = std::iter::empty()
            .chain(module_specs.iter().map(|(name, _, _)| name))
            .chain(output_names.iter().copied())
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();

        let modules = module_specs
            .iter()
            .map(|(name, kind, outputs_to)| {
                let name = name.to_string();
                let kind = *kind;
                let outputs_to = outputs_to
                    .iter()
                    .map(|output_name| name_to_index.get(output_name).unwrap())
                    .cloned()
                    .collect();
                Module {
                    name,
                    kind,
                    outputs_to,
                }
            })
            .chain(output_names.iter().map(|name| {
                let name = name.to_string();
                Module {
                    name,
                    kind: ModuleKind::Output,
                    outputs_to: Vec::new(),
                }
            }))
            .chain(std::iter::once(Module {
                name: "button".to_string(),
                kind: ModuleKind::Button,
                outputs_to: vec![name_to_index
                    .get(&"broadcaster")
                    .unwrap()
                    .clone()],
            }))
            .collect();

        Ok(System { modules })
    }

    fn part_1(
        system: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        // let mut initial_state = system.initial_state();
        // let num_pulses = initial_state
        //     .process_button_presses(1)
        //     .inspect(|pulse| {
        //         let Pulse {
        //             kind,
        //             sender,
        //             receiver,
        //         } = pulse;
        //         let sender = &system.modules[*sender].name;
        //         let receiver = &system.modules[*receiver].name;
        //         println!("Pulse {kind} from {sender} to {receiver}");
        //     })
        //     .map(|pulse| pulse.kind)
        //     .counts();
        // println!("Num pulses: {num_pulses:?}");

        let mut initial_state = system.initial_state();
        let product = initial_state
            .process_button_presses(1000)
            .map(|pulse| pulse.kind)
            .counts()
            .into_values()
            .product::<usize>();

        Ok(product)
    }

    fn part_2(
        system: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        // system
        //     .find_indirect_inputs()
        //     .into_iter()
        //     .sorted_by_key(|(to, _)| *to)
        //     .map(|(to, froms)| {
        //         let to = system.get_name(to).unwrap();
        //         let froms = froms
        //             .into_iter()
        //             .map(|from| system.get_name(from).unwrap())
        //             .join(", ");
        //         (to, froms)
        //     })
        //     .for_each(|(to, froms)| {
        //         println!("{to} <= {froms}");
        //     });

        let rx = system
            .modules
            .iter()
            .enumerate()
            .find(|(_, module)| module.name == "rx")
            .map(|(i, _)| i)
            .expect("No 'rx' module found");

        // The rx nodes is the conjunction of several independent
        // inputs.  Those inputs should each have their own cycle?
        let cycle = system
            .modules
            .iter()
            .enumerate()
            .filter(|(_, module)| {
                module.outputs_to.iter().any(|output| *output == rx)
            })
            .flat_map(|(i, _)| {
                system
                    .modules
                    .iter()
                    .enumerate()
                    .filter(move |(_, module)| {
                        module.outputs_to.iter().any(|output| *output == i)
                    })
                    .map(|(i, _)| i)
            })
            .inspect(|i| println!("{}", system.get_name(*i).unwrap()))
            .map(|i| system.find_cycle(i))
            .inspect(|i| println!("\t{i:?}"))
            .map(|(a, b)| b - a)
            .product::<usize>();

        Ok(cycle)

        // let mut state = system.initial_state();
        // let num_presses = (0..)
        //     .inspect(|i| {
        //         if i % 10000 == 0 {
        //             println!("Checked up through {i}");
        //         }
        //     })
        //     .flat_map(|i| {
        //         state
        //             .process_single_pulse(button_press)
        //             .map(|pulse| (i, pulse))
        //             .collect::<Vec<_>>()
        //             .into_iter()
        //     })
        //     .find(|(_, pulse)| {
        //         pulse.kind == PulseKind::Low && pulse.receiver == rx
        //     })
        //     .map(|(i, _)| i)
        //     .unwrap();
        // Ok(num_presses)
    }
}
