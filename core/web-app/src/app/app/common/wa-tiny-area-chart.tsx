import { Area, AreaChart, ResponsiveContainer } from 'recharts';

export default function WATinyAreaChart({
  data,
  yDataKey,
}: {
  data: any;
  yDataKey: string;
}) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <AreaChart data={data}>
        <defs>
          <linearGradient id="areaColor" x1="0" y1="0" x2="0" y2="1">
            <stop offset="60%" stopColor="#3354FF" stopOpacity={0.5} />
            <stop offset="100%" stopColor="#FFFFFF" stopOpacity={0.5} />
          </linearGradient>
        </defs>
        <Area
          type="basis"
          dataKey={yDataKey}
          stroke="#8884d8"
          fill="url(#areaColor)"
        />
      </AreaChart>
    </ResponsiveContainer>
  );
}
