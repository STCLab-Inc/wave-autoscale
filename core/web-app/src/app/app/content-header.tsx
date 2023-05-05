import classNames from 'classnames';
import { ReactNode } from 'react';

export default function ContentHeader({
  title,
  children,
  right,
}: {
  title: string;
  children?: ReactNode;
  right?: ReactNode;
}) {
  return (
    <div
      className={classNames(
        'h-22 prose flex w-full min-w-full flex-col border-b border-base-300 bg-slate-50 px-3 pt-3',
        {
          'pb-3': !children,
        }
      )}
    >
      {/* Plan Title */}
      <div className="flex-start flex">
        <h3 className="m-3 flex-1">{title}</h3>
        {right}
      </div>
      {children}
    </div>
  );
}
