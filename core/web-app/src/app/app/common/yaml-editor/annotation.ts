import { YAMLException } from 'js-yaml';

export function getAnnotationsFromError(error: any) {
  if (
    error instanceof YAMLException &&
    error.mark?.column &&
    error.mark?.line &&
    error.message
  ) {
    return [
      {
        row: error.mark.line,
        column: error.mark.column,
        text: error.message,
        type: 'error',
      },
    ];
  }
  return [];
}
