import { DataLayer } from '@/infra/data-layer';
import YAML from 'yaml';

class DefinitionServiceClass {
  async createDefinitions(yaml: string) {
    try {
      YAML.parseAllDocuments(yaml);
    } catch (e) {
      throw new Error('Invalid YAML');
    }
    const response = await DataLayer.post('/api/definitions', {
      yaml,
    });
    if (response.status !== 200) {
      const errorMessage = response.data;
      throw new Error(errorMessage);
    }
    return response.data;
  }
}

const DefinitionService = new DefinitionServiceClass();

export default DefinitionService;
