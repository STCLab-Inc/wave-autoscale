import { DataLayer } from '@/infra/data-layer';
import StatsService from '../stats';

class MetricServiceClass {
  async getMetrics() {
    const response = await DataLayer.get('/api/metrics');
    return response.data;
  }
  async getMetricYamls() {
    const response = await DataLayer.get('/api/metrics/yaml');
    return response.data;
  }
  async syncMetricYaml(yaml: string) {
    const response = await DataLayer.post('/api/metrics/yaml', {
      yaml,
    });
    await StatsService.invalidateStats();
    return response.data;
  }
}

const MetricService = new MetricServiceClass();

export default MetricService;
