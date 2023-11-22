import classNames from 'classnames';
import React from 'react';

interface RenderKeyValuePairsProps {
  [key: string]: any;
}

export function renderKeyValuePairs(
  keyValuePairs: RenderKeyValuePairsProps,
  indent?: boolean,
  depth = 0
): React.ReactNode {
  const indentFlag = indent ?? true;

  return (
    <div
      className={classNames('w-full pb-2', {
        'pl-4': indentFlag && depth > 0,
      })}
    >
      {Object.keys(keyValuePairs)
        .sort()
        .flatMap((key) => {
          const value = keyValuePairs[key];

          if (value != null && typeof value === 'object') {
            return (
              <div key={key}>
                <div className="whitespace-normal break-all font-bold">
                  {key}
                </div>
                {renderKeyValuePairs(value, indentFlag, depth + 1)}
              </div>
            );
          }
          if (!value) {
            return null;
          }

          return (
            <div key={key}>
              <div className="whitespace-normal break-all font-bold">{key}</div>
              <div className="whitespace-normal break-all">{value}</div>
            </div>
          );
        })}
    </div>
  );
}

export function renderKeyValuePairsWithJson(
  jsonString: string,
  indent?: boolean
): React.ReactNode {
  const indentFlag = indent ?? true;

  console.log(jsonString);

  try {
    const keyValuePairs = JSON.parse(jsonString);
    return renderKeyValuePairs(keyValuePairs, indentFlag);
  } catch (error) {
    /* console.log({ error }); */
  }

  return indentFlag ? (
    <div>{jsonString}</div>
  ) : (
    <div className="w-full pb-2">
      <div className="whitespace-normal break-all ">{jsonString}</div>
    </div>
  );
}
