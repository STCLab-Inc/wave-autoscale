import PlanService from '@/services/plan';
import { PlanItemDefinition } from '@/types/bindings/plan-item-definition';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { enableMapSet, produce } from 'immer';
import { create } from 'zustand';

enableMapSet();

interface ScalingPlanState {
  modifiedAt?: Date;
  savedAt?: Date;
  scalingPlan?: ScalingPlanDefinition;
  plans: PlanItemDefinition[];
  selectedPlan: PlanItemDefinition | undefined;
}

interface PlanState {
  // Cache
  scalingPlanMap: Map<string, ScalingPlanState>;
  currentScalingPlanState?: ScalingPlanState;
  // Scaling Plan
  fetch: (scalingPlanId: string) => Promise<void>;
  // Plan Items
  getPlanItems: (
    scalingPlanId: string
  ) => Promise<PlanItemDefinition[] | undefined>;
  addPlanItem: (scalingPlanId: string) => Promise<void>;
  updatePlanItem: (
    scalingPlanId: string,
    plan: PlanItemDefinition
  ) => Promise<void>;
  updatePlanItemUI: (
    scalingPlanId: string,
    plan: PlanItemDefinition
  ) => Promise<void>;
  clearSelectedPlan: (scalingPlanId: string) => Promise<void>;
  needToSave: (scalingPlanId: string) => boolean;
}

const ID_PREFIX = 'plan_';

export const usePlanStore = create<PlanState>((set, get) => ({
  // Cache
  // modifiedAt: undefined,
  // savedAt: undefined,
  // currentScalingPlanId: undefined,
  // scalingPlan: undefined,
  // plans: [],
  // selectedPlan: undefined,

  scalingPlanMap: new Map<string, ScalingPlanState>(),
  currentScalingPlanState: undefined,
  // Fetch from server to get the latest plan
  fetch: async (scalingPlanId: string) => {
    const state = get();
    let scalingPlanState = state.scalingPlanMap.get(scalingPlanId);
    if (!scalingPlanState) {
      const scalingPlan = await PlanService.getPlan(scalingPlanId);
      scalingPlanState = {
        modifiedAt: undefined,
        savedAt: undefined,
        scalingPlan,
        plans: scalingPlan?.plans || [],
        selectedPlan: scalingPlan?.plans.find(
          (p: PlanItemDefinition) => p.ui?.selected
        ),
      };
      set(
        produce((state: PlanState) => {
          if (scalingPlanState) {
            state.scalingPlanMap.set(scalingPlanId, scalingPlanState);
          }
        })
      );
    }
    set(
      produce((state: PlanState) => {
        state.currentScalingPlanState = scalingPlanState;
      })
    );
  },
  getPlanItems: async (scalingPlanId: string) => {
    await get().fetch(scalingPlanId);
    let scalingPlanState = get().scalingPlanMap.get(scalingPlanId);
    return scalingPlanState?.plans;
  },
  addPlanItem: async (scalingPlanId: string) => {
    // Fetch from server to get the latest plan
    await get().fetch(scalingPlanId);
    set(
      await produce((state: PlanState) => {
        const scalingPlanState = state.scalingPlanMap.get(scalingPlanId);
        if (!scalingPlanState) {
          return;
        }
        let postfixNumber = scalingPlanState.plans.length + 1;
        let id = `${ID_PREFIX}${postfixNumber}`;
        while (scalingPlanState.plans.find((plan) => plan.id === id)) {
          postfixNumber++;
          id = `${ID_PREFIX}${postfixNumber}`;
        }
        scalingPlanState.plans = [
          ...scalingPlanState.plans,
          {
            id,
            ui: {},
            description: '',
            expression: '',
            scaling_components: [],
            priority: 0,
          },
        ];
        scalingPlanState.modifiedAt = new Date();
        state.currentScalingPlanState = scalingPlanState;
      })
    );
  },
  updatePlanItem: async (scalingPlanId: string, plan: PlanItemDefinition) => {
    // Fetch from server to get the latest plan
    await get().fetch(scalingPlanId);
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.scalingPlanMap.get(scalingPlanId);
        if (!scalingPlanState) {
          return;
        }
        const plans = scalingPlanState.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.id === plan.id
        );
        if (index === -1) {
          return;
        }
        plans[index] = plan;
        scalingPlanState.modifiedAt = new Date();
        state.currentScalingPlanState = scalingPlanState;
      })
    );
  },
  updatePlanItemUI: async (scalingPlanId: string, plan: PlanItemDefinition) => {
    // Fetch from server to get the latest plan
    await get().fetch(scalingPlanId);
    set(
      produce((state: PlanState) => {
        const scalingPlanState = state.scalingPlanMap.get(scalingPlanId);
        if (!scalingPlanState) {
          return;
        }
        const plans = scalingPlanState.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.id === plan.id
        );
        if (index === -1) {
          return;
        }
        plans[index] = plan;
        const selectedPlan = plans.find(
          (p: PlanItemDefinition) => p.ui?.selected
        );
        scalingPlanState.selectedPlan = selectedPlan;
        scalingPlanState.modifiedAt = new Date();
        state.currentScalingPlanState = scalingPlanState;
      })
    );
  },
  clearSelectedPlan: async (scalingPlanId: string) => {
    // Fetch from server to get the latest plan
    await get().fetch(scalingPlanId);
    set(
      produce((state) => {
        const scalingPlanState = state.scalingPlanMap.get(scalingPlanId);
        if (!scalingPlanState) {
          return;
        }
        const plans = scalingPlanState.plans;
        const index = plans.findIndex(
          (p: PlanItemDefinition) => p.ui?.selected
        );
        if (index !== -1) {
          plans[index].ui.selected = false;
        }
        scalingPlanState.selectedPlan = undefined;
        scalingPlanState.modifiedAt = new Date();
        state.currentScalingPlanState = scalingPlanState;
      })
    );
  },
  needToSave: (scalingPlanId: string) => {
    const state = get();
    const scalingPlanState = state.scalingPlanMap.get(scalingPlanId);
    if (!scalingPlanState) {
      // throw new Error('Scaling plan not found');
      return false;
    }

    if (!scalingPlanState.modifiedAt) {
      return false;
    }
    if (!scalingPlanState.savedAt) {
      return true;
    }
    return scalingPlanState.modifiedAt <= scalingPlanState.savedAt;
  },
}));
