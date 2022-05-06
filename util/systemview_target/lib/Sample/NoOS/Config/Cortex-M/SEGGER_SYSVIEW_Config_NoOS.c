#include "SEGGER_SYSVIEW.h"
#include "SEGGER_SYSVIEW_Conf.h"

extern void _rtos_trace_system_description(void);
extern void _rtos_trace_task_list(void);
extern long long unsigned int _rtos_trace_time(void);
extern unsigned int _rtos_trace_sysclock(void);


// The application name to be displayed in SystemViewer
#define SYSVIEW_APP_NAME        "Rust Application"

// The target device name
#define SYSVIEW_DEVICE_NAME     "Cortex-M4"

// The lowest RAM address used for IDs (pointers)
#define SYSVIEW_RAM_BASE        (0x00000000)

// Define as 1 if the Cortex-M cycle counter is used as SystemView timestamp. Must match SEGGER_SYSVIEW_Conf.h
#ifndef   USE_CYCCNT_TIMESTAMP
  #define USE_CYCCNT_TIMESTAMP    1
#endif

// Define as 1 if the Cortex-M cycle counter is used and there might be no debugger attached while recording.
#ifndef   ENABLE_DWT_CYCCNT
  #define ENABLE_DWT_CYCCNT       (USE_CYCCNT_TIMESTAMP & SEGGER_SYSVIEW_POST_MORTEM_MODE)
#endif


#define DEMCR                     (*(volatile unsigned long*) (0xE000EDFCuL))   // Debug Exception and Monitor Control Register
#define TRACEENA_BIT              (1uL << 24)                                   // Trace enable bit
#define DWT_CTRL                  (*(volatile unsigned long*) (0xE0001000uL))   // DWT Control Register
#define NOCYCCNT_BIT              (1uL << 25)                                   // Cycle counter support bit
#define CYCCNTENA_BIT             (1uL << 0)                                    // Cycle counter enable bit


static void send_system_description(void) {
    _rtos_trace_system_description();
    SEGGER_SYSVIEW_SendTaskList();
}


static SEGGER_SYSVIEW_OS_API os_callbacks = {
        .pfGetTime = _rtos_trace_time,
        .pfSendTaskList = _rtos_trace_task_list,
};

/*********************************************************************
*
*       Global functions
*
**********************************************************************
*/
void SEGGER_SYSVIEW_Conf(void) {
#if USE_CYCCNT_TIMESTAMP
#if ENABLE_DWT_CYCCNT
  //
  // If no debugger is connected, the DWT must be enabled by the application
  //
  if ((DEMCR & TRACEENA_BIT) == 0) {
    DEMCR |= TRACEENA_BIT;
  }
#endif
  //
  //  The cycle counter must be activated in order
  //  to use time related functions.
  //
  if ((DWT_CTRL & NOCYCCNT_BIT) == 0) {       // Cycle counter supported?
    if ((DWT_CTRL & CYCCNTENA_BIT) == 0) {    // Cycle counter not enabled?
      DWT_CTRL |= CYCCNTENA_BIT;              // Enable Cycle counter
    }
  }
#endif
  SEGGER_SYSVIEW_Init(
          _rtos_trace_sysclock(),
          _rtos_trace_sysclock(),
          &os_callbacks,
          send_system_description);
  SEGGER_SYSVIEW_SetRAMBase(SYSVIEW_RAM_BASE);
}