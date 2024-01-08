import { DataLayer } from '@/infra/data-layer';
import { Dayjs } from 'dayjs';

class InflowServiceClass {
  async getInflowMetricId() {
    const response = await DataLayer.get(`/api/inflow/metric_id`);
    return response.data;
  }
  async getInflowWithMetricIdByDate(
    metric_id: String,
    from: Dayjs,
    to: Dayjs
  ) {
    const response = await DataLayer.get(
      `/api/inflow?metric_id=${metric_id}&from=${
        from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
      }&to=${to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}`
    );
    return response.data;
  }
}

const InflowService = new InflowServiceClass();

export default InflowService;
