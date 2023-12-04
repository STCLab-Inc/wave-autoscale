import React, { ReactNode } from 'react';

import classNames from 'classnames';

type ContentHeaderType = 'MAIN' | 'SUB';

interface ContentHeaderProps {
  type?: ContentHeaderType;
  title?: string;
  children?: ReactNode;
  right?: ReactNode;
}

function getContentHeaderClassNames(type?: string) {
  switch (type) {
    case 'SUB':
      return classNames(
        'flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-t border-gray-200'
      );
    case 'MAIN':
    default:
      return classNames(
        'flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75'
      );
  }
}

function getContentHeaderTitleClassNames(type?: string) {
  switch (type) {
    case 'SUB':
      return classNames(
        'font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000'
      );
    case 'MAIN':
    default:
      return classNames(
        'font-Pretendard truncate whitespace-nowrap px-8 text-lg font-semibold text-gray-1000'
      );
  }
}

function getContentHeaderRightClassNames(type?: string) {
  switch (type) {
    case 'SUB':
      return classNames('flex px-4');
    case 'MAIN':
    default:
      return classNames('flex px-8');
  }
}

export default function ContentHeader({
  type,
  title,
  children,
  right,
}: ContentHeaderProps) {
  return (
    <div className={getContentHeaderClassNames(type)}>
      <div className={getContentHeaderTitleClassNames(type)}>{title}</div>
      <div className={getContentHeaderRightClassNames(type)}>{right}</div>
      {children}
    </div>
  );
}
