import classNames from 'classnames';

export function renderKeyValuePairs(
  keyValuePairs: { [key: string]: any },
  indent?: boolean,
  depth = 0
) {
  const indentFlag = indent ?? true;

  return Object.keys(keyValuePairs)
    .sort()
    .map((key) => {
      let value = keyValuePairs[key];
      if (value != null && typeof value === 'object') {
        value = renderKeyValuePairs(value, indentFlag, 1);
      }

      if (Array.isArray(value)) {
        return (
          <div
            key={key}
            className={classNames('mb-2 w-full', {
              'ml-4': indentFlag,
            })}
          >
            <div className="whitespace-normal break-all">{value}</div>
          </div>
        );
      }

      if (!value) {
        return null;
      }

      return (
        <div
          key={key}
          className={classNames('mb-2 w-full', {
            'ml-4': indentFlag,
          })}
        >
          <div className="whitespace-normal break-all font-bold">{key}</div>
          <div className="whitespace-normal break-all">{value}</div>
        </div>
      );
    });
}

export function renderKeyValuePairsWithJson(
  jsonString: string,
  indent?: boolean
) {
  const indentFlag = indent ?? true;

  try {
    const keyValuePairs = JSON.parse(jsonString);
    return renderKeyValuePairs(keyValuePairs, indentFlag);
  } catch (error) {
    /* console.log({ error }); */
  }
  return indentFlag ? (
    <div>{jsonString}</div>
  ) : (
    <div className="mb-2 w-full">
      <div className="whitespace-normal break-all ">{jsonString}</div>
    </div>
  );
}
