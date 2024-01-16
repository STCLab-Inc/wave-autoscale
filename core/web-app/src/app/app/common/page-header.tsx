interface PageHeaderProps {
  title: string;
  subtitle?: string;
}
export default function PageHeader(props: PageHeaderProps) {
  return (
    <div className="flex h-14 min-h-14 w-full items-center border-b border-wa-gray-200 bg-white px-6">
      <div className="text-[1rem] font-semibold">{props.title}</div>
      {props.subtitle && (
        <>
          {/* divider */}
          <div className="mx-[10px] h-6 w-[1px] bg-wa-gray-300" />
          <div className="text-sm text-wa-gray-600">{props.subtitle}</div>
        </>
      )}
    </div>
  );
}
