interface PageHeaderProps {
  title: string;
}
export default function PageHeader(props: PageHeaderProps) {
  return <div className="flex h-14 min-h-14 w-full items-center bg-white px-6">
    <div className="text-[1rem] font-semibold">{props.title}</div>
  </div>;
}
