import { DataLayer } from '@/infra/data-layer';
import { Dayjs } from 'dayjs';

class InflowServiceClass {
  async getInflowByFromTo(from: Dayjs, to: Dayjs) {
    /* TODO */
    const response = await DataLayer.get(
      `/api/inflow?from=${from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}&to=${
        to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
      }`
    );
    return response.data;
  }
}

const InflowService = new InflowServiceClass();

export default InflowService;
