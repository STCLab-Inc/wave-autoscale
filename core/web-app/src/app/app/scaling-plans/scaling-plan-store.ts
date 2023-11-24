import { enableMapSet, produce } from 'immer';
import { create } from 'zustand';
import * as acorn from 'acorn';
import * as walk from 'acorn-walk';
import { parse as parseYaml, stringify as stringifyYaml } from 'yaml';

import ScalingPlanService from '@/services/scaling-plan';
import { ScalingPlansItemDefinition } from '@/types/bindings/scaling-plans-item-definition';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

enableMapSet();

export interface MetricUI {
  id: string;
  selected?: boolean;
}
export interface ScalingComponentUI {
  id: string;
  selected?: boolean;
}

interface ScalingPlansState {
  // The Plan has PlanItems
  scalingPlan: ScalingPlanDefinition;
  // For UI
  selectedScalingPlansItem?: ScalingPlansItemDefinition;
  modifiedAt?: Date;
  savedAt?: Date;
}

interface ScalingPlanState {
  // Cache
  cache: Map<string, ScalingPlansState>;
  // Get current scaling plan. Do not modify it directly
  currentScalingPlanState?: ScalingPlansState;
  currentScalingPlanId?: string;
  // Load from server. It should be called before any other actions
  load: (scalingPlanId: string) => Promise<void>;
  // Save to server
  push: () => Promise<void>;
  // Get current scaling plan state
  getCurrentScalingPlanState: (state?: any) => ScalingPlansState;
  // For internal use
  syncCurrentScalingPlan: () => void;
  // For current scaling plan
  addScalingPlanItem: () => void;
  updateScalingPlanItem: (scalingPlan: ScalingPlansItemDefinition) => void;
  updateScalingPlanItemUI: (scalingPlan: ScalingPlansItemDefinition) => void;
  removeScalingPlanItem: (scalingPlanId: string) => void;
  clearSelection: () => void;
  // General actions, not related to current scaling plan
  needToSave: (scalingPlanId: string) => boolean;
  getYAMLCode: () => string;
  applyYAMLCode: (yamlCode: string) => void;
}

export const useScalingPlanStore = create<ScalingPlanState>((set, get) => ({
  // Cache
  cache: new Map<string, ScalingPlansState>(),
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
      const scalingPlan = await ScalingPlanService.getScalingPlan(
        scalingPlanId
      );
      scalingPlanState = {
        scalingPlan,
        modifiedAt: undefined,
        savedAt: undefined,
        selectedScalingPlansItem: undefined,
      };
      // Save to cache
      set(
        produce((state: ScalingPlanState) => {
          if (scalingPlanState) {
            state.cache.set(scalingPlanId, scalingPlanState);
          }
        })
      );
    }
    // Set current scaling plan
    set(
      produce((state: ScalingPlanState) => {
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
      const response = await ScalingPlanService.updateScalingPlan(scalingPlan);
      // TODO: Handle response
      set(
        produce((state: ScalingPlanState) => {
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
      produce((state: ScalingPlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const scalingPlans = scalingPlanState.scalingPlan.plans;
        // Update metric ids and scaling component ids
        scalingPlans.forEach((scalingPlan) => {
          const expression = scalingPlan.expression;
          const metricIdsInScalingPlan = new Set<string>();
          const scalingComponentIdsInScalingPlan = new Set<string>();
          if (expression) {
            // 1. Metric Ids
            try {
              const ast = acorn.parse(expression, {
                ecmaVersion: 2020,
                sourceType: 'script',
              });
              walk.simple(ast, {
                ObjectExpression(node: any) {
                  // Check if this ObjectExpression is part of a BinaryExpression
                  const metricIdProperty = node.properties.find(
                    (property: any) => {
                      return (
                        property.key.type === 'Identifier' &&
                        property.key.name === 'metric_id'
                      );
                    }
                  );

                  if (
                    metricIdProperty &&
                    metricIdProperty.value.type === 'Literal'
                  ) {
                    const metricIdValue = metricIdProperty.value.value;
                    metricIdsInScalingPlan.add(metricIdValue);
                  }
                },
              });
            } catch (error) {
              metricIdsInScalingPlan.add('Not found');
              // TODO: Error handling
            }
          }

          // 2. Scaling Component Ids
          scalingPlan.scaling_components?.forEach((component) => {
            scalingComponentIdsInScalingPlan.add(component.component_id);
          });
          scalingPlan.ui = {
            ...(scalingPlan.ui ?? {}),
            metrics: Array.from(metricIdsInScalingPlan).map(
              (metricId) =>
                ({
                  id: metricId,
                  selected: false,
                } as MetricUI)
            ),
            scalingComponents: Array.from(scalingComponentIdsInScalingPlan).map(
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
  addScalingPlanItem: () => {
    set(
      produce((state: ScalingPlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const scalingPlans = scalingPlanState.scalingPlan.plans;
        // Generate new id
        const ID_PREFIX = 'scaling_plans_';
        let postfixNumber = scalingPlans.length + 1;
        let id = `${ID_PREFIX}${postfixNumber}`;
        // Find a unique id
        while (scalingPlans.find((scalingPlan) => scalingPlan.id === id)) {
          postfixNumber++;
          id = `${ID_PREFIX}${postfixNumber}`;
        }
        scalingPlanState.scalingPlan.plans = [
          ...scalingPlans,
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
  updateScalingPlanItem: (scalingPlan: ScalingPlansItemDefinition) => {
    set(
      produce((state: ScalingPlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const scalingPlans = scalingPlanState.scalingPlan.plans;
        const index = scalingPlans.findIndex(
          (p: ScalingPlansItemDefinition) => p.id === scalingPlan.id
        );
        if (index === -1) {
          throw new Error(`Plan item ${scalingPlan.id} not found`);
        }
        scalingPlans[index] = {
          ...scalingPlans[index],
          ...scalingPlan,
        };
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },
  updateScalingPlanItemUI: (scalingPlan: ScalingPlansItemDefinition) => {
    set(
      produce((state: ScalingPlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const scalingPlans = scalingPlanState.scalingPlan.plans;
        const index = scalingPlans.findIndex(
          (p: ScalingPlansItemDefinition) => p.id === scalingPlan.id
        );
        if (index === -1) {
          throw new Error(`Plan item ${scalingPlan.id} not found`);
        }
        scalingPlans[index] = scalingPlan;
        const selectedScalingPlan = scalingPlans.find(
          (p: ScalingPlansItemDefinition) => p.ui?.selected
        );
        scalingPlanState.selectedScalingPlansItem = selectedScalingPlan;
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },
  removeScalingPlanItem: (scalingPlanId: string) => {
    set(
      produce((state: ScalingPlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const scalingPlans = scalingPlanState.scalingPlan.plans;
        const index = scalingPlans.findIndex(
          (p: ScalingPlansItemDefinition) => p.id === scalingPlanId
        );
        if (index === -1) {
          throw new Error(`Scaling plan item ${scalingPlanId} not found`);
        }
        scalingPlans.splice(index, 1);
        scalingPlanState.modifiedAt = new Date();
      })
    );
    get().syncCurrentScalingPlan();
  },

  clearSelection: () => {
    set(
      produce((state: ScalingPlanState) => {
        const scalingPlanState = state.getCurrentScalingPlanState(state);
        const scalingPlans = scalingPlanState.scalingPlan.plans;
        const index = scalingPlans.findIndex(
          (p: ScalingPlansItemDefinition) => p.ui?.selected
        );
        if (index !== -1) {
          scalingPlans[index].ui.selected = false;
        }
        scalingPlanState.selectedScalingPlansItem = undefined;
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
      draft.plans?.forEach((scalingPlan: any) => {
        delete scalingPlan.ui;
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
        produce((state: ScalingPlanState) => {
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
