import classNames from 'classnames';

export function renderKeyValuePairs(
  keyValuePairs: { [key: string]: any },
  indent?: boolean,
  depth = 0
) {
  const indentFlag = indent === undefined || indent === true ? true : false;

  return Object.keys(keyValuePairs)
    .sort()
    .map((key) => {
      let value = keyValuePairs[key];
      if (value != null && typeof value === 'object') {
        value = renderKeyValuePairs(value, indentFlag, 1);
      }
      if (!value) {
        return null;
      }

      return indentFlag ? (
        <div
          key={key}
          className={classNames('mb-2', {
            'ml-4': depth > 0,
          })}
        >
          <div className="whitespace-normal break-all font-bold">{key}</div>
          <div className="whitespace-normal break-all ">{value}</div>
        </div>
      ) : (
        <div
          key={key}
          className={classNames('mb-2 w-full', {
            'ml-0': depth > 0,
          })}
        >
          <div className="whitespace-normal break-all font-bold">{key}</div>
          <div className="whitespace-normal break-all ">{value}</div>
        </div>
      );
    });
}

export function renderKeyValuePairsWithJson(
  jsonString: string,
  indent?: boolean
) {
  const indentFlag = indent === undefined || indent === true ? true : false;

  try {
    const keyValuePairs = JSON.parse(jsonString);
    return renderKeyValuePairs(keyValuePairs, indentFlag);
  } catch (error) {
    console.log({ error });
  }
  return indentFlag ? (
    <div>{jsonString}</div>
  ) : (
    <div className={classNames('mb-2 w-full')}>
      <div className="whitespace-normal break-all ">{jsonString}</div>
    </div>
  );
}
