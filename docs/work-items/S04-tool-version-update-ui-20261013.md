# S04: 도구 버전 확인·선택 UI

- 상태: in-progress
- 분류: feature
- 우선순위: P1
- 요구사항: R02, R08, R11
- 담당/수정 범위: advanced_units — 새 tool_update_ui.rs 후보; root — catalog/Main/worker 통합
- 선행 작업: 기존 공통 UI·읽기 전용 inspector·update-flow backend; 제품 소스 채택은 root 인수 후

## 사용자 결과

도구 버전 확인·선택 UI를 기존 Windows 네이티브 흐름에 연결하고 실제 실패/표시 동작을 검증한다.

## 계약과 범위

후보는 `.local-tools/session-recovery-20261013/` 새 namespace에 보존한다. 같은 파일을 동시에 수정하지 않으며 root가 공유 계약·제품 source freeze·통합 검증을 조정한다.

## 인수 조건

native typed intent만 전달. 최신 관측을 설치 권한으로 취급하지 않으며 정확한 버전/target/manifest/generation을 유지한다. 호버·pressed·focus·선택·오류·CloseWait·200% 좁은 배치를 실제 Context로 검증한다.

## 검증과 handoff

v1은 ID 변환 test-only 컴파일 실패를 보존했다. v2는 그 변환만 수정해 root가 Windows 실제 회귀를 실행 중이다. 제품 UI 미연결이며 최신 설치 성공/실제 HTTP/물리 GUI 인수를 주장하지 않는다.

## 2026-10-13 실제 실행 결과

Windows MSVC 후보 `flow-ui-v2`에서 호버·pressed·focus·선택·설치 불가·CloseWait·200% 배치 회귀 5개가 통과했다. root가 원본 로그, 소유 Job 자연 종료 및 source/candidate 불변을 확인했다. 미연결 상태의 unused variant 경고 3개는 남아 있으며 제품 통합 완료나 전체 UI 인수를 뜻하지 않는다.

## 2026-10-14 통합 checkpoint

renderer5/0, receipt3/0, 캐시28/0, 업데이트 실제 dispatcher/channel/poll10/0, 변경 목록·helper·타임라인50/0 및 workspace all-target strict clippy를 격리 확인했다. 최초 업데이트9/1은 dispatcher를 우회한 fixture serial0 불일치로 보존했고 실제 Check 경로로 수정하여 기대(2,1)를 유지했다. v21 개발 소스에 통합했으며 fresh whole 진행 중이다. Deno/FFmpeg 관측된 unknown 최신 버전의 dynamic 설치, 실제 네트워크 공급/정식 라이선스·서명 인수는 남는다.
