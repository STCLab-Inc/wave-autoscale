import React from 'react';

import classNames from 'classnames';

interface RenderKeyValuePairsProps {
  [key: string]: any;
}

export function renderKeyValuePairs(
  keyValuePairs: RenderKeyValuePairsProps,
  indent?: boolean,
  depth = 0,
  preventIndent?: boolean
): React.ReactNode {
  const indentFlag = indent ?? true;
  const preventIndentFlag = preventIndent ?? false;

  return (
    <div
      className={classNames('w-full', {
        'pl-4':
          indentFlag &&
          depth > 0 &&
          !Array.isArray(keyValuePairs) &&
          !preventIndentFlag,
      })}
    >
      {Object.keys(keyValuePairs)
        .sort()
        .flatMap((key, index) => {
          const value = keyValuePairs[key];

          if (value != null && typeof value === 'object') {
            return Array.isArray(keyValuePairs) ? (
              renderKeyValuePairs(value, indentFlag, depth + 1, true)
            ) : (
              <div key={index} className="flex w-full flex-col">
                <div className="flex w-full flex-col whitespace-normal break-all font-bold">
                  {key}
                </div>
                {renderKeyValuePairs(value, indentFlag, depth + 1)}
              </div>
            );
          }

          if (!value) {
            return null;
          }

          return Array.isArray(keyValuePairs) ? (
            <div key={key} className="flex w-full flex-col">
              <span className="flex w-full flex-col whitespace-normal break-all">
                {value}
              </span>
            </div>
          ) : (
            <div key={key} className="flex w-full flex-col">
              <span className="hitespace-normal flex w-full flex-col break-all font-bold">
                {key}
              </span>
              <span className="flex w-full flex-col whitespace-normal break-all">
                {value}
              </span>
            </div>
          );
        })}
    </div>
  );
}

export function renderKeyValuePairsWithJson(
  data: object | string,
  indent?: boolean
): React.ReactNode {
  const indentFlag = indent ?? true;

  if (typeof data === 'object') {
    data = JSON.stringify(data);
  }

  try {
    const keyValuePairs = JSON.parse(data);
    return renderKeyValuePairs(keyValuePairs, indentFlag);
  } catch (error) {
    /* console.error({ error }); */
  }

  return indentFlag ? (
    <span>{data}</span>
  ) : (
    <div className="w-full pb-2">
      <div className="whitespace-normal break-all ">{data}</div>
    </div>
  );
}
