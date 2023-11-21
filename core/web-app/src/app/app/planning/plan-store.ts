import PlanService from '@/services/plan';
import { PlanItemDefinition } from '@/types/bindings/plan-item-definition';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { enableMapSet, produce } from 'immer';
import { create } from 'zustand';
import * as acorn from 'acorn';
import * as walk from 'acorn-walk';
import { parse as parseYaml, stringify as stringifyYaml } from 'yaml';

enableMapSet();

export interface MetricUI {
  id: string;
  selected?: boolean;
}
export interface ScalingComponentUI {
  id: string;
  selected?: boolean;
}

interface ScalingPlanState {
  // The Plan has PlanItems
  scalingPlan: ScalingPlanDefinition;
  // For UI
  selectedPlanItem?: PlanItemDefinition;
  modifiedAt?: Date;
  savedAt?: Date;
}

interface PlanState {
  // Cache
  cache: Map<string, ScalingPlanState>;
  // Get current scaling plan. Do not modify it directly
  currentScalingPlanState?: ScalingPlanState;
  currentScalingPlanId?: string;
  // Load from server. It should be called before any other actions
  load: (scalingPlanId: string) => Promise<void>;
  // Save to server
  push: () => Promise<void>;
  // Get current scaling plan state
  getCurrentScalingPlanState: (state?: any) => ScalingPlanState;
  // For internal use
  syncCurrentScalingPlan: () => void;
  // For current scaling plan
  addPlanItem: () => void;
  updatePlanItem: (plan: PlanItemDefinition) => void;
  updatePlanItemUI: (plan: PlanItemDefinition) => void;
  removePlanItem: (planId: string) => void;
  clearSelection: () => void;
  // General actions, not related to current scaling plan
  needToSave: (scalingPlanId: string) => boolean;
  getYAMLCode: () => string;
  applyYAMLCode: (yamlCode: string) => void;
}

export const usePlanStore = create<PlanState>((set, get) => ({
  // Cache
  cache: new Map<string, ScalingPlanState>(),
  currentScalingPlanState: undefined,
  currentScalingPlanId: undefined,

  // Fetch from server to get the latest plan
  load: async (scalingPlanId: string) => {
    const state = get();
    // Load from cache
    let scalingPlanState = state.cache.get(scalingPlanId);
    // If there is no cache
    if (!scalingPlanState) {
      // Load from server
      const scalingPlan = await PlanService.getPlan(scalingPlanId);
      scalingPlanState = {
        scalingPlan,
        modifiedAt: undefined,
        savedAt: undefined,
        selectedPlanItem: undefined,
      };
      // Save to cache
      set(
        produce((state: PlanState) => {
          if (scalingPlanState) {
            state.cache.set(scalingPlanId, scalingPlanState);
          }
        })
      );
    }
    // Set current scaling plan
    set(
      produce((state: PlanState) => {
        state.currentScalingPlanState = scalingPlanState;
        state.currentScalingPlanId = scalingPlanId;
      })
    );
    state.syncCurrentScalingPlan();
  },
  push: async () => {
    const state = get();
    const scalingPlanState = state.getCurrentScalingPlanState();
    const scalingPlan = scalingPlanState.scalingPlan;
    try {
      const response = await PlanService.updatePlan(scalingPlan);
      // TODO: Handle response
      set(
        produce((state: PlanState) => {
          const scalingPlanState = state.getCurrentScalingPlanState(state);
          scalingPlanState.savedAt = new Date();
        })
      );
    } catch (error) {
      // TODO: Handle error
    }
    state.syncCurrentScalingPlan();
  },
  getCurrentScalingPlanState: (state?: any) => {
    // if state is not provided from produce(immer), get the latest state
    if (!state) {
      state = get();
    }
    if (!state.currentScalingPlanId) {
      throw new Error(`Should call load() before getCurrentState()`);
    }
    const scalingPlanState = state.cache.get(state.currentScalingPlanId);
    if (!scalingPlanState || !scalingPlanState.scalingPlan) {
      throw new Error(`Should call load() before push()`);
    }
    return scalingPlanState;
  },
  syncCurrentScalingPlan: () => {
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const plans = scalingPlanState.scalingPlan.plans;
        // Update metric ids and scaling component ids
        plans.forEach((plan) => {
          const expression = plan.expression;
          const metricIdsInPlan = new Set<string>();
          const scalingComponentIdsInPlan = new Set<string>();
          if (expression) {
            // 1. Metric Ids
            try {
              // Abstract Syntax Tree
              const ast = acorn.parse(expression, {
                ecmaVersion: 2020,
                sourceType: 'script',
              });
              walk.simple(ast, {
                Identifier(node: any) {
                  metricIdsInPlan.add(node.name);
                },
              });
            } catch (error) {
              // TODO: Error handling
            }
          }

          // 2. Scaling Component Ids
          plan.scaling_components?.forEach((component) => {
            scalingComponentIdsInPlan.add(component.component_id);
          });

          plan.ui = {
            ...(plan.ui ?? {}),
            metrics: Array.from(metricIdsInPlan).map(
              (metricId) =>
                ({
                  id: metricId,
                  selected: false,
                } as MetricUI)
            ),
            scalingComponents: Array.from(scalingComponentIdsInPlan).map(
              (scalingComponentId) =>
                ({
                  id: scalingComponentId,
                  selected: false,
                } as ScalingComponentUI)
            ),
          };
        });

        state.currentScalingPlanState = scalingPlanState;
      })
    );
  },
  addPlanItem: () => {
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const plans = scalingPlanState.scalingPlan.plans;
        // Generate new id
        const ID_PREFIX = 'plan_';
        let postfixNumber = plans.length + 1;
        let id = `${ID_PREFIX}${postfixNumber}`;
        // Find a unique id
        while (plans.find((plan) => plan.id === id)) {
          postfixNumber++;
          id = `${ID_PREFIX}${postfixNumber}`;
        }
        scalingPlanState.scalingPlan.plans = [
          ...plans,
          {
            id,
            ui: {},
            description: '',
            expression: '',
            cron_expression: null,
            scaling_components: [],
            priority: 0,
          },
        ];
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },
  updatePlanItem: (plan: PlanItemDefinition) => {
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const plans = scalingPlanState.scalingPlan.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.id === plan.id
        );
        if (index === -1) {
          throw new Error(`Plan item ${plan.id} not found`);
        }
        plans[index] = {
          ...plans[index],
          ...plan,
        };
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },
  updatePlanItemUI: (plan: PlanItemDefinition) => {
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const plans = scalingPlanState.scalingPlan.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.id === plan.id
        );
        if (index === -1) {
          throw new Error(`Plan item ${plan.id} not found`);
        }
        plans[index] = plan;
        const selectedPlan = plans.find(
          (p: PlanItemDefinition) => p.ui?.selected
        );
        scalingPlanState.selectedPlanItem = selectedPlan;
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },
  removePlanItem: (planId: string) => {
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const plans = scalingPlanState.scalingPlan.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.id === planId
        );
        if (index === -1) {
          throw new Error(`Plan item ${planId} not found`);
        }
        plans.splice(index, 1);
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },

  clearSelection: () => {
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const plans = scalingPlanState.scalingPlan.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.ui?.selected
        );
        if (index !== -1) {
          plans[index].ui.selected = false;
        }
        scalingPlanState.selectedPlanItem = undefined;
      })
    );
    get().syncCurrentScalingPlan();
  },
  needToSave: (scalingPlanId: string) => {
    const state = get();
    const scalingPlanState = state.cache.get(scalingPlanId);
    if (!scalingPlanState) {
      // throw new Error('Scaling plan not found');
      return false;
    }

    if (!scalingPlanState.modifiedAt) {
      return false;
    } else if (!scalingPlanState.savedAt) {
      return true;
    }
    return scalingPlanState.modifiedAt > scalingPlanState.savedAt;
  },
  getYAMLCode: () => {
    const state = get();
    const scalingPlanState = state.getCurrentScalingPlanState(state);
    const scalingPlan = scalingPlanState.scalingPlan;
    const scalingPlanForCode = produce(scalingPlan, (draft: any) => {
      delete draft.db_id;
      draft.plans?.forEach((plan: any) => {
        delete plan.ui;
      });
    });
    const code = stringifyYaml(scalingPlanForCode);
    return code;
  },
  applyYAMLCode: (code: string) => {
    const state = get();
    try {
      const newScalingPlan = parseYaml(code);
      set(
        produce((state: PlanState) => {
          const scalingPlanState = state.getCurrentScalingPlanState(state);
          const dbId = scalingPlanState.scalingPlan.db_id;
          scalingPlanState.scalingPlan = newScalingPlan;
          scalingPlanState.scalingPlan.db_id = dbId;
          scalingPlanState.modifiedAt = new Date();
        })
      );
      get().syncCurrentScalingPlan();
    } catch (error) {
      // TODO: Error handling\
      throw error;
    }
  },
}));
