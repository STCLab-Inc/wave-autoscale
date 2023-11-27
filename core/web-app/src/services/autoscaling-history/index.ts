import { DataLayer } from '@/infra/data-layer';
import { Dayjs } from 'dayjs';

class AutoscalingHistoryServiceClass {
  async getAutoscalingHistoryByFromTo(from: Dayjs, to: Dayjs) {
    /* TODO */
    const response = await DataLayer.get(
      `/api/autoscaling-history?from=${
        from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
      }&to=${to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}`
    );
    return response.data;
  }
}

const AutoscalingHistoryService = new AutoscalingHistoryServiceClass();

export default AutoscalingHistoryService;
