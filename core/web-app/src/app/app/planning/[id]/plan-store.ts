import { produce } from 'immer';
import { create } from 'zustand';

export interface Plan {
  id: string;
  description?: string;
  expression?: string;
  priority?: number;
  ui?: object;
}
interface PlanState {
  plans: Plan[];
  addPlan: () => void;
  updatePlan: (plan: Plan) => void;
}

const ID_PREFIX = 'scale-plan-';
export const usePlanStore = create<PlanState>((set, get) => ({
  plans: [],
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
        plans[index] = plan;
      })
    ),
}));
