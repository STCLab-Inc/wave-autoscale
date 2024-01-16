import { DataLayer } from '@/infra/data-layer';
import { MetricDefinition } from '@/types/bindings/metric-definition';

class MetricServiceClass {
  async getMetrics() {
    const response = await DataLayer.get('/api/metrics');
    return response.data;
  }
  async getMetric(db_id: string) {
    const response = await DataLayer.get(`/api/metrics/${db_id}`);
    return response.data;
  }
  async createMetric(metric: MetricDefinition) {
    const response = await DataLayer.post('/api/metrics', {
      metrics: [metric],
    });
    return response.data;
  }
  async updateMetric(metric: MetricDefinition) {
    if (!metric.db_id) {
      throw new Error('Metric does not have a db_id');
    }

    const response = await DataLayer.put(
      `/api/metrics/${metric.db_id}`,
      metric
    );
    return response.data;
  }
  async deleteMetric(db_id: string) {
    const response = await DataLayer.delete(`/api/metrics/${db_id}`);
    return response.data;
  }
}

const MetricService = new MetricServiceClass();

export default MetricService;
