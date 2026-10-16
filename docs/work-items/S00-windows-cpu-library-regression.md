# S00 Windows CPU Library snapshot regression

- 상태: completed
- 우선순위: P0
- 요구사항: R01, R08, R11
- 담당: guardian_fault
- 수정 파일: 전용 work-item/validation/evidence; windows_cpu_probe.ps1 필요시만 독점
- 의존성: root frozen Windows CPU SHA38e7d11ad92b77011a562c679cf08a42a2c7d4fbab8e69931768da5baa7086c0,5438464B
- 인수: inactive privateDesktop startup/render, owned synthetic download Cancel/StopExit 실제 회귀; HWND/PID/worker start/Job/cleanup/inputdesktop unchanged; raw JSON/PNG 보존; source/binary before-after 동일
- 제한: 사용자 desktop/clipboard/global input/fallback 금지. 제품/Cargo/shared scripts 수정 금지. 이전5cb1 증거 보존. Library tab 실증 가능성은 별도 확인하며 UI 행동 없이는 완료 주장 금지.

## 실제 결과

새hash startup/render/Cancel/빈Library 탭 확인 완료. StopExit는 두 실제시도 모두 입력 후 Start에서 URL필요 오류/worker미시작이므로 미검증. 원시 typed/after-start PNG와 exit1 JSON 보존; 제품원인단정금지. 제품3소스/binary before-afterMATCH 후 root freeze해제. 8h시험 유지. 상세 전용 validation 참조.

후속입력가설대조로 StopExit실증완료: matchedkeys trial exit0/workerowned,startmatch,cleanup/primary0. 이전미검증2회실패는불변보존. 관련 S00-windows-cpu-library-input.md와validation참조.
