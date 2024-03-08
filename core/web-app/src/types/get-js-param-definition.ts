export interface GetJSParamDefinition {
  metricId: string;
  name?: string;
  tags?: {
    [key: string]: string;
  };
  period?: string;
  stats?: string;
}
