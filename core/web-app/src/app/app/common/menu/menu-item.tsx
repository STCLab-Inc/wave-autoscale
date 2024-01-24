import classNames from 'classnames';
import Link from 'next/link';
import DefinitionCountBadge from '../definition-count-badge';

interface MenuItemProps {
  iconUrl: string;
  label: string;
  targetPath: string;
  badgeCount?: number;
  selected?: boolean;
}
export default function MenuItem({
  iconUrl,
  label,
  targetPath,
  badgeCount,
  selected,
}: MenuItemProps) {
  return (
    <Link href={targetPath} className="w-full">
      <li
        className={classNames(
          'flex h-12 w-full flex-row items-center justify-start px-6',
          {
            'bg-wa-blue-50': selected,
          }
        )}
      >
        {/* Icon */}
        <div className="mr-2 h-6 w-6">
          <img src={iconUrl} alt={label} className="max-h-full max-w-full" />
        </div>
        {/* Label */}
        <span className="ml-2 text-sm text-wa-gray-700">{label}</span>
        {/* Badge Count */}
        {(badgeCount !== undefined) && (
          <DefinitionCountBadge count={badgeCount} />
        )}
      </li>
    </Link>
  );
}
