import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';

export default function WAAreaChart({
  data,
  yDataKey = 'y',
  xDataKey = 'x',
  syncId = 'syncId',
  simpleXAxis,
  xTickFormatter,
}: {
  data: any;
  yDataKey: string;
  xDataKey: string;
  syncId?: string;
  simpleXAxis?: boolean;
  xTickFormatter?: (value: any) => string;
}) {
  return (
    <ResponsiveContainer width="100%" height="100%">
      <AreaChart
        data={data}
        margin={{
          top: 0,
          right: 0,
          left: 0,
          bottom: 0,
        }}
        syncId={syncId}
      >
        <defs>
          <linearGradient id="areaColor" x1="0" y1="0" x2="0" y2="1">
            <stop offset="60%" stopColor="#3354FF" stopOpacity={0.5} />
            <stop offset="100%" stopColor="#FFFFFF" stopOpacity={0.5} />
          </linearGradient>
        </defs>
        <XAxis
          dataKey={xDataKey}
          style={{
            fontSize: '0.75rem',
          }}
          tickFormatter={xTickFormatter}
          tickLine={!simpleXAxis}
          tick={!simpleXAxis}
        />

        <YAxis
          dataKey={yDataKey}
          style={{
            fontSize: '0.75rem',
          }}
        />
        <Tooltip />
        <CartesianGrid strokeDasharray="3 3" />

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
