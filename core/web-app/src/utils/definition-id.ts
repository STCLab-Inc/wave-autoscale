export function transformDefinitionId(id: string | undefined): string {
  let result = id?.replace(/[^a-z0-9_]/g, '') ?? '';
  result = result.replace(/^[0-9]/, '');
  return result;
}

export const DEFINITION_ID_RULE_DESCRIPTION =
  '(alphanumeric, lowercase and underscore)';
