import { DataLayer } from '@/infra/data-layer';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

class ScalingComponentServiceClass {
  async getScalingComponents() {
    const response = await DataLayer.get('/api/scaling-components', {});
    return response.data;
  }
  async getScalingComponent(db_id: string) {
    const response = await DataLayer.get(`/api/scaling-components/${db_id}`);
    return response.data;
  }
  async createScalingComponent(component: ScalingComponentDefinition) {
    const response = await DataLayer.post('/api/scaling-components', {
      scaling_components: [component],
    });
    return response.data;
  }
  async updateScalingComponent(component: ScalingComponentDefinition) {
    if (!component.db_id) {
      throw new Error('ScalingComponent does not have a db_id');
    }

    const response = await DataLayer.put(
      `/api/scaling-components/${component.db_id}`,
      component
    );
    return response.data;
  }
  async deleteScalingComponent(db_id: string) {
    const response = await DataLayer.delete(`/api/scaling-components/${db_id}`);
    return response.data;
  }
}

const ScalingComponentService = new ScalingComponentServiceClass();

export default ScalingComponentService;
