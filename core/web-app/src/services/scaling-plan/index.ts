import { DataLayer } from '@/infra/data-layer';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

class ScalingPlanServiceClass {
  async getScalingPlans() {
    const response = await DataLayer.get('/api/plans');
    return response.data;
  }

  async getScalingPlan(id: string) {
    const response = await DataLayer.get(`/api/plans/${id}`);
    return response.data;
  }

  async createScalingPlan(plan: ScalingPlanDefinition) {
    const response = await DataLayer.post('/api/plans', {
      plans: [plan],
    });
    return response.data;
  }

  async updateScalingPlan(plan: ScalingPlanDefinition) {
    const response = await DataLayer.put(`/api/plans/${plan.db_id}`, plan);
    return response.data;
  }

  async deleteScalingPlan(id: string) {
    const response = await DataLayer.delete(`/api/plans/${id}`);
    return response.data;
  }
}

const ScalingPlanService = new ScalingPlanServiceClass();

export default ScalingPlanService;
