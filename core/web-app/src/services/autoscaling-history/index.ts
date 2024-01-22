import { DataLayer } from '@/infra/data-layer';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import { Dayjs } from 'dayjs';

class AutoscalingHistoryServiceClass {
  async getAutoscalingHistoryByFromTo(
    from: Dayjs,
    to: Dayjs
  ): Promise<AutoscalingHistoryDefinition[]> {
    /* TODO */
    const { data } = await DataLayer.get(
      `/api/autoscaling-history?from=${
        from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
      }&to=${to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}`
    );
    return data;
  }
}

const AutoscalingHistoryService = new AutoscalingHistoryServiceClass();

export default AutoscalingHistoryService;
