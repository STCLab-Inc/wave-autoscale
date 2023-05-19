import { DataLayer } from '@/infra/data-layer';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

class PlanServiceClass {
  async getPlans() {
    const response = await DataLayer.get('/api/plans');
    return response.data;
  }

  async getPlan(id: string) {
    const response = await DataLayer.get(`/api/plans/${id}`);
    return response.data;
  }

  async createPlan(plan: ScalingPlanDefinition) {
    const response = await DataLayer.post('/api/plans', {
      plans: [plan],
    });
    return response.data;
  }

  async updatePlan(plan: ScalingPlanDefinition) {
    const response = await DataLayer.put(`/api/plans/${plan.db_id}`, plan);
    return response.data;
  }
}

const PlanService = new PlanServiceClass();

export default PlanService;
