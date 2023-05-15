import { produce } from 'immer';
import { create } from 'zustand';

export interface Plan {
  id: string;
  description?: string;
  expression?: string;
  priority?: number;
  ui?: any;
}
interface PlanState {
  plans: Plan[];
  selectedPlan: Plan | undefined;
  addPlan: () => void;
  updatePlan: (plan: Plan) => void;
}

const ID_PREFIX = 'scale-plan-';
export const usePlanStore = create<PlanState>((set, get) => ({
  plans: [],
  selectedPlan: undefined,
  addPlan: () =>
    set(
      produce((state: PlanState) => {
        let postfixNumber = state.plans.length + 1;
        let id = `${ID_PREFIX}${postfixNumber}`;
        while (state.plans.find((plan) => plan.id === id)) {
          postfixNumber++;
          id = `${ID_PREFIX}${postfixNumber}`;
        }
        return { plans: [...state.plans, { id }] };
      })
    ),
  updatePlan: (plan: Plan) =>
    set(
      produce((state) => {
        const plans = state.plans;
        const index = plans.findIndex((p: Plan) => p.id === plan.id);
        if (index === -1) return;
        plans[index] = plan;
        const selectedPlan = plans.find((p: Plan) => p.ui?.selected);
        state.selectedPlan = selectedPlan;
      })
    ),
}));
