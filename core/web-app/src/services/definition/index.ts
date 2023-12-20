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
    return response.data;
  }
}

const DefinitionService = new DefinitionServiceClass();

export default DefinitionService;
