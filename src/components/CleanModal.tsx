import React, { useState } from 'react'
import { Button, Group, Modal, Stack, Switch, Text } from '@mantine/core'
import { IconDownload, IconEraser, IconRestore } from '@tabler/icons-react'
import { type CleanOptions, DEFAULT_CLEAN_OPTIONS } from '@/types/metadata'
import classes from './CleanModal.module.css'

interface CleanModalProps {
  opened: boolean
  onClose: () => void
  onCleanAndDownload: (options: CleanOptions) => void
  loading?: boolean
}

export const CleanModal: React.FC<CleanModalProps> = ({
  opened,
  onClose,
  onCleanAndDownload,
  loading = false,
}) => {
  const [options, setOptions] = useState<CleanOptions>(DEFAULT_CLEAN_OPTIONS)

  const handleToggle = (key: keyof CleanOptions) => {
    setOptions((prev) => ({
      ...prev,
      [key]: !prev[key],
    }))
  }

  const handleReset = () => {
    setOptions(DEFAULT_CLEAN_OPTIONS)
  }

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={
        <Group gap="xs">
          <IconEraser size={20} color="#6366f1" />
          <Text fw={700}>Custom Privacy & Metadata Stripping Settings</Text>
        </Group>
      }
      size="md"
      centered
      radius="lg"
    >
      <Stack gap="xs">
        <Text size="xs" c="dimmed" mb="xs">
          Configure which metadata segments to remove from the image losslessly
          without re-compressing pixel bitstreams.
        </Text>

        <div className={classes.optionRow}>
          <div className={classes.optionText}>
            <span className={classes.optionLabel}>Strip All Non-Essential</span>
            <span className={classes.optionDesc}>
              Removes all ancillary tags, prompts, comments, and EXIF
            </span>
          </div>
          <Switch
            checked={options.strip_all}
            onChange={() => handleToggle('strip_all')}
            color="indigo"
          />
        </div>

        <div className={classes.optionRow}>
          <div className={classes.optionText}>
            <span className={classes.optionLabel}>
              Strip AI Prompts & Workflows
            </span>
            <span className={classes.optionDesc}>
              Removes Stable Diffusion, ComfyUI, NovelAI prompt chunks
            </span>
          </div>
          <Switch
            checked={options.strip_ai_metadata}
            onChange={() => handleToggle('strip_ai_metadata')}
            color="indigo"
          />
        </div>

        <div className={classes.optionRow}>
          <div className={classes.optionText}>
            <span className={classes.optionLabel}>Strip GPS Geolocation</span>
            <span className={classes.optionDesc}>
              Removes GPS coordinates, altitude, and timestamps
            </span>
          </div>
          <Switch
            checked={options.strip_gps}
            onChange={() => handleToggle('strip_gps')}
            color="indigo"
          />
        </div>

        <div className={classes.optionRow}>
          <div className={classes.optionText}>
            <span className={classes.optionLabel}>Strip EXIF Camera Data</span>
            <span className={classes.optionDesc}>
              Removes hardware serial, lens model, shutter/ISO info
            </span>
          </div>
          <Switch
            checked={options.strip_exif}
            onChange={() => handleToggle('strip_exif')}
            color="indigo"
          />
        </div>

        <div className={classes.optionRow}>
          <div className={classes.optionText}>
            <span className={classes.optionLabel}>
              Strip C2PA Content Credentials
            </span>
            <span className={classes.optionDesc}>
              Removes JUMBF assertions and digital publishing signatures
            </span>
          </div>
          <Switch
            checked={options.strip_c2pa}
            onChange={() => handleToggle('strip_c2pa')}
            color="indigo"
          />
        </div>

        <div className={classes.optionRow}>
          <div className={classes.optionText}>
            <span className={classes.optionLabel}>
              Preserve ICC Color Profile
            </span>
            <span className={classes.optionDesc}>
              Keep color accuracy (Recommended: OFF to keep profile, ON to
              strip)
            </span>
          </div>
          <Switch
            checked={!options.strip_icc_profile}
            onChange={() => handleToggle('strip_icc_profile')}
            color="indigo"
          />
        </div>

        <Group justify="space-between" mt="lg">
          <Button
            variant="subtle"
            color="gray"
            size="xs"
            leftSection={<IconRestore size={14} />}
            onClick={handleReset}
          >
            Reset Defaults
          </Button>

          <Group gap="xs">
            <Button variant="subtle" color="gray" onClick={onClose}>
              Cancel
            </Button>
            <Button
              className={classes.cleanSubmitBtn}
              loading={loading}
              leftSection={<IconDownload size={16} />}
              onClick={() => onCleanAndDownload(options)}
            >
              Lossless Strip & Download
            </Button>
          </Group>
        </Group>
      </Stack>
    </Modal>
  )
}
