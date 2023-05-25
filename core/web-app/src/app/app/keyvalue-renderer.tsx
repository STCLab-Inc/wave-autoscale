import classNames from 'classnames';

export function renderKeyValuePairs(
  keyValuePairs: { [key: string]: any },
  depth = 0
) {
  return Object.keys(keyValuePairs)
    .sort()
    .map((key) => {
      let value = keyValuePairs[key];
      if (value != null && typeof value === 'object') {
        value = renderKeyValuePairs(value, 1);
      }
      if (!value) {
        return null;
      }
      return (
        <div
          key={key}
          className={classNames('max-w-sm break-all', {
            'ml-4': depth > 0,
          })}
        >
          <span className="font-bold">{key}</span>:{' '}
          <span className="break-all">{value}</span>
        </div>
      );
    });
}

export function renderKeyValuePairsWithJson(jsonString: string) {
  try {
    const keyValuePairs = JSON.parse(jsonString);
    return renderKeyValuePairs(keyValuePairs);
  } catch (error) {
    console.log({ error });
  }
  return <div>{jsonString}</div>;
}
