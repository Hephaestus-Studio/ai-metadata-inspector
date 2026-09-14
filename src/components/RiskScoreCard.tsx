import React from 'react'
import { Badge, Box, Group, Stack, Text } from '@mantine/core'
import { IconCheck, IconShieldCheck, IconShieldX } from '@tabler/icons-react'
import clsx from 'clsx'
import type { RiskReport } from '@/types/metadata'
import classes from './RiskScoreCard.module.css'

interface RiskScoreCardProps {
  riskReport: RiskReport
  embedded?: boolean
}

export const RiskScoreCard: React.FC<RiskScoreCardProps> = ({
  riskReport,
  embedded = false,
}) => {
  const { score, level, items } = riskReport

  const isSafe = level === 'safe'
  const isMedium = level === 'medium'

  const scoreClass = isSafe
    ? classes.safeScore
    : isMedium
      ? classes.mediumScore
      : classes.highScore

  const levelBadgeColor = isSafe ? 'teal' : isMedium ? 'orange' : 'red'
  const levelText = isSafe
    ? 'Safe to Share'
    : isMedium
      ? 'Medium Exposure'
      : 'High Privacy Risk'

  return (
    <div className={clsx(classes.card, embedded && classes.cardEmbedded)}>
      <Group justify="space-between" align="center" wrap="nowrap">
        <Group gap="sm" wrap="nowrap">
          <div className={clsx(classes.scoreCircle, scoreClass)}>
            <span>{score}</span>
          </div>

          <div>
            <Group gap={6} wrap="nowrap">
              <Text size="sm" fw={700}>
                Privacy Score
              </Text>
              <Badge color={levelBadgeColor} variant="light" size="xs">
                {levelText}
              </Badge>
            </Group>
            <Text size="xs" c="dimmed" mt={2} lineClamp={1}>
              {isSafe
                ? 'Minimal tracking metadata.'
                : 'Contains GPS or device serials.'}
            </Text>
          </div>
        </Group>

        {isSafe ? (
          <IconShieldCheck size={26} color="#10b981" />
        ) : (
          <IconShieldX size={26} color={isMedium ? '#f59e0b' : '#ef4444'} />
        )}
      </Group>

      {/* Identified Risk Items */}
      {items.length > 0 ? (
        <Stack gap={6} mt="xs">
          <Text size="xs" fw={700} c="dimmed" tt="uppercase" mt={4}>
            Detected Exposure ({items.length}):
          </Text>
          {items.map((item, index) => {
            const isHigh = item.severity === 'high'
            const isMed = item.severity === 'medium'
            const dotColor = isHigh ? '#ef4444' : isMed ? '#f59e0b' : '#38bdf8'

            return (
              <div key={index} className={classes.riskItem}>
                <div
                  className={classes.severityDot}
                  style={{ backgroundColor: dotColor }}
                />
                <Box style={{ flex: 1, minWidth: 0 }}>
                  <Group justify="space-between" align="baseline" wrap="nowrap">
                    <Text size="xs" fw={700} truncate>
                      {item.title}
                    </Text>
                    <Badge
                      size="xs"
                      variant="outline"
                      color={isHigh ? 'red' : isMed ? 'orange' : 'cyan'}
                    >
                      {item.category}
                    </Badge>
                  </Group>
                  <Text size="xs" c="dimmed" lineClamp={1}>
                    {item.description}
                  </Text>
                </Box>
              </div>
            )
          })}
        </Stack>
      ) : (
        <Group gap={6} mt="xs" c="teal">
          <IconCheck size={14} />
          <Text size="xs" fw={600}>
            No sensitive tracking tags found.
          </Text>
        </Group>
      )}
    </div>
  )
}
