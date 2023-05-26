import { DataLayer } from '@/infra/data-layer';
import { Dayjs } from 'dayjs';

class AutoscalingHistoryServiceClass {
  async getHistoryByFromTo(from: Dayjs, to: Dayjs) {
    const response = await DataLayer.get(
      `/api/autoscaling-history?from=${from.toISOString()}&to=${to.toISOString()}`
    );
    return response.data;
  }
}

const AutoscalingHistoryService = new AutoscalingHistoryServiceClass();

export default AutoscalingHistoryService;
