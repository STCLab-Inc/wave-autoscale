export function generateGetCode({
  metricId,
  name,
  tags,
}: {
  metricId: string;
  name?: string;
  tags?: any;
}) {
  let code = `get({ metric_id: '${metricId}'`;

  if (name) {
    code += `, name: '${name}'`;
  }

  if (tags) {
    code += `, tags: ${JSON.stringify(tags)}`;
  }

  code += `})`;

  return code;
}
