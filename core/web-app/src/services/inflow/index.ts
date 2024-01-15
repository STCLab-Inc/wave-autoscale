import { DataLayer } from '@/infra/data-layer';
import { InflowLogItem } from '@/types/inflow-log';

class InflowServiceClass {
  async getInflowMetricIds() {
    const response = await DataLayer.get(`/api/inflow/metric_id`);
    return response.data;
  }
  async getInflowMetricLogs(
    metricId: String,
    count: number
  ): Promise<InflowLogItem[]> {
    const { data } = await DataLayer.get(
      `/api/inflow?metric_id=${metricId}&count=${count}`
    );
    const logs: InflowLogItem[] = [];
    data.map((log: { json_value: string }, index: number) => {
      const data: InflowLogItem[] = JSON.parse(log.json_value);
      logs.push(...data);
    });
    return logs;
  }
}

const InflowService = new InflowServiceClass();

export default InflowService;
