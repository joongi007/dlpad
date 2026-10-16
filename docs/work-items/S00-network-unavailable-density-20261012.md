# S00: 네트워크 미지원 안내의 정보 밀도

- 상태: in-progress
- 분류: quality
- 우선순위: P0
- 요구사항: R08, R13
- 담당/수정 범위: 주 담당 catalog.rs, accessible_metrics.rs 및 이 문서; 보조 담당은 /tmp 후보와 관측기 준비만 수행
- 선행 작업: tools-scroll-checkbox whole 및 현재 GUI 원본 독립 검토 완료

## 사용자 결과

네트워크 관측 설정을 펼치면 짧은 미지원 상태를 먼저 보여주고, 다운로드 속도와 저장 설정에 관한 자세한 설명은 명시적으로 펼칠 수 있다. 높은 배율에서 긴 설명이 입력 영역을 밀어내는 기본 화면의 정보 밀도를 줄인다.

## 계약과 범위

PRODUCT의 간단한 기본값과 DESIGN의 점진 공개를 따른다. Windows의 입력·적용 비활성 정책과 저장값, Linux 동작, worker와 one-shot reveal을 유지한다. 공유 schema, Cargo 및 의존성 변경은 없다. 전 화면에 모든 설정이 동시에 보인다는 계약을 추가하지 않는다.

## 인수 조건

- 실제 Context에서 기본 짧은 상태, 설명 펼침·접힘, 저장값 및 비활성 정책을 확인한다.
- 최신 Windows 빌드와 가상 화면의 150/200%에서 기본 설명 및 입력 영역의 표시·스크롤 접근성을 직접 확인한다.
- 전체 설명이 보존되고 펼친 뒤 보통 스크롤로 접근 가능해야 한다.

## 검증과 handoff

변경 전 current-ui-tools-scroll-checkbox-20261012의 실제 150/200 설정 캡처를 직접 검토했다. 200%에서는 긴 설명이 온전히 보이지만 입력·적용은 현재 viewport 아래였다. 원본 수집 exit0과 독립 reviewer exit0, 141개 캡처 및 두 private cleanup을 확인했다. 이는 원본/관측 의미 검증이며 전체 픽셀 인수는 아니다.

변경 후 Windows CPU/MSVC whole fmt·clippy·test·release는 1,139개 성공/0 실패/30 ignored이며 원본과 소스·실행 파일 pin을 독립 확인했다. 실제 Context의 설명 펼침·접힘 및 저장값/비활성 유지 회귀가 포함된다. 두 번의 통합 compile 실패는 별도 원본 증거에 보존했고 corrected-whole만 성공 근거로 사용한다.

current-ui-network-density-20261012는 149개 캡처와 두 private 실행의 원본/receipt를 독립 검토했다. 모든 캡처를 시각 인수한 것은 아니다. 별도 current-ui-network-scroll-transport-20261012는 두 private 실행 자연 종료와 focused 원본 검토를 통과했다. 주 담당은 Light/Dark 150%의 전체 설명·입력·적용 표시, Light/Dark 200%의 스크롤 후 입력과 전체 설명이 하단 상태줄 위에 보이는 것을 직접 확인하고 root pixel review JSON에 기록했다. 200% 적용 버튼 접근·물리 마우스·IME·전체 UI는 이 검증 범위에 포함하지 않는다.

이전 focused observer는 WM_MOUSEWHEEL만 전송해 200% 설명 접근 시험에 실패했다. 원본 실패를 보존하고 vendored winit/egui의 pointer 위치 요구에 따라 별도 관측기에서 owned HWND의 WM_MOUSEMOVE 후 wheel을 전송했다. 제품 UI의 스크롤 정책을 우회하거나 visible desktop 입력을 사용하지 않았다. 이 결과는 합성 native 입력 범위이며 실제 물리 입력과 다르다.

같은 고정 빌드의 current-ui-network-density-idle-cost-20261012는 3회 × 60초 및 18개 고정 구간 비용 검증과 총 30,000개 fixture payload/SQLite 검토를 통과했다. 최대 단일 코어 CPU는 0.624765% 미만이며 WS/private 각 80MiB 미만이다. quiet host 또는 실제 사용자 미디어/모든 상태 성능을 보장하지 않는다. 이후 placeholder 수정은 별도 snapshot이다.

작업 상태는 확대 화면의 적용 버튼 및 전체 사용자 UI/UX 인수가 남아 in-progress로 유지한다.
