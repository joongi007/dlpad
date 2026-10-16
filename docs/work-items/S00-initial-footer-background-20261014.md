# S00: 초기 화면의 footer 위 빈 검은 영역

- 상태: in-progress
- 분류: bug-investigation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: windows_auto — 독립 재현/후보; root — accessible.rs/software_host.rs 통합·검증
- 선행 작업: v21 source664 GUI63 직접 검토

## 사용자 결과

재시작 초기 frame에서 footer 위에 검은 빈 영역이 나타나지 않고, 상태 행 높이 변화와 확대/축소에서 배경이 현재 테마로 채워져야 한다.

## 계약과 범위

frozen v21 gui/restart/before-uia.png에서 y573..612 검은 띠를 실제 관측했다. 이후 settled capture에는 없으며 원인을 아직 단정하지 않는다. software_host는 non-reused buffer.fill(0), renderer 내부 canvas도0clear한다. dynamic panel 크기의 미도색 영역 가설을 실제 Candidate CPU raster/동일 init 변화로 검증한다. Root가 P1 Main/Settings merge를 담당하며 후보는 독립 namespace only. 새 renderer/framework/전체속도 최적화 개편은 제외한다.

## 인수 조건

원사진/소스/재현FAIL보존. 실제 Root Context/CPU 그림에서 테마·status 높이 변경·첫 frame·resize의 빈 배경을 확인하고 수정후회귀. 직접 UI 픽셀 검토는 unit PASS로 대체하지 않는다. 본문의 일반적인 스크롤로 잘리는 offscreen 요소와 기능적 input full-clip 실패를 구분한다.

## 검증과 handoff

Root11(+추가1)와 child52이미지 실제 검토, black band와200% theme combo clip을 관측했다. 최초 실행 재현/수정/strict/performance와 새 native first/settled 캡처는 남아 있다. 실제Frame/CPU 측정 없이 성능 비용없음 또는 UI 완료로 보고하지 않는다.

2026-10-14 실제 후보 검증: feedback-background-raster-red-v1은104 actualCPU raster records에서5,253,120 alpha0 pixel/0PASS1FAIL을 재현했다. 배경 wrapper 첫 수정은 일부만 해결하여2,457,600 alpha0/0PASS1FAIL로 보존했다. residual은 test initial200의 pendingzoom이 첫 viewport를5000×5000으로 바꾸는 경로였고 real Appearance API 별도RED에서도480×320 대신5000×5000을 관측했다. 첫pass만 immediate 옵션 초기화하고 이후deferred를 유지한 green-v2의104 raster는alpha0=0, 실제viewport×pp==target을 통과했다. strict는 시험 closure의불필요drop 경고로 실패했다. 해당 실행 끝 guard 전에 Root가 시험파일 수정 타이밍을 겹쳐 source핀 최종값은 통합 인수로 사용하지 않고 고정소스 재검사한다. 새 location-settings 통합 검사에 같은 시험을 포함했다. UI 실제가상화면 첫/settled현재제품 확인과 비용측정은 아직남아있다.
