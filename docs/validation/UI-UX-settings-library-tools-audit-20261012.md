# 설정·파일 목록·도구 상호작용 검증 잔여

읽기 독립 감사. 프로젝트/native/Cargo 실행 없음. 현재 소스 검토와 과거 캡처를 분리하며 전체 UI 인수 완료를 뜻하지 않는다.

기준: DESIGN.md의 32 logical 입력, Light/Dark 동등성, 800창 접근성. GTMUX Input.svelte 76–115의 12px padding, hover border, focus-visible outline, disabled opacity, 오류 field 연결은 상태 구별 기준이다. 웹 구현 도입 제안은 아니다.

| 우선 | 실제 사용자 행동 / 남은 인수 | 현재 근거와 한계 |
|---|---|---|
| 1 | 파일 목록에서 긴 파일명을 선택한 뒤 포인터 hover 없이 키보드만으로 전체 경로 확인. Tab/Arrow→Enter와 선택/포커스가 서로 구별되어야 한다. | accessible_library.rs 365–405는 truncate + on_hover_text, 안정 path ID, 새 포커스 one-shot reveal를 구현한다. 이미 해결된 128행 Tab/Arrow를 다시 결함으로 쓰지 않는다. 전체 경로 tooltip의 키보드 공개 및 selected+focus 동시 시각 상태는 이 감사가 직접 확인한 캡처로 증명되지 않는다. 행을 모두 생성하는 해결안은 필요 없다. |
| 2 | 도구 설치를 시작한 뒤 busy 중 취소로 이동하고 실패 후 재시도: 사용자가 선택한 도구, 일하는 상태, 비활성 이유, 해당 도구 오류를 한 화면에서 구별. | accessible_tools.rs 346–378은 busy 동안 탭 비활성, 462–505는 설치/취소/오류를 표시한다. 이미 인수한 3탭 설치 접근/본문 ScrollArea/Dark200를 중복 잔여로 쓰지 않는다. 설치 도중 selected 탭+disabled+focus+오류의 실제 combined-state 캡처는 여기서 확인하지 않았다. 순수 worker 정책 PASS가 해당 시각 인수는 아니다. |
| 3 | Windows 설정에서 네트워크 도움말 펼침/닫힘 후 원 저장값 유지, disabled 입력/Apply에 실제 입력 시도 없이 미지원 이유 이해. Linux에서는 잘못된 alias→오류→정상 복귀 시 caret/값 유지. | accessible_metrics.rs 411–461은 비Linux disabled와 이유-first, 원 값을 변경하지 않음. 이미 해결한 one-shot 공개/도움말 density를 재요청하지 않는다. Windows의 측정 불가 표시는 fallback 계약이며 NIC 구현·실제 관측 완료를 뜻하지 않는다. 현재 Windows API 후보가 있으므로 제품 NIC 연결·검증을 별도 P0 잔여로 유지한다. Linux enabled/error/caret의 실제 OS GUI 증거는 이번 Windows 감사 범위 밖이다. |

## 실제 본 화면

파일 목록 Light 800/200 empty after: 빈 목록 안내와 도움말은 footer 위에서 읽힌다. 폴더 입력은 위로 스크롤되어 일부만 보인다. 이것은 안내 공개 캡처의 의도된 scroll 상태이며 입력 잘림 회귀로 단정하지 않는다. disabled ‘다음 페이지’가 보인다. 빈 상태 사진으로 populated 행 hover/selected/focus 인수를 주장하지 않는다.

이 세 항목은 재현된 제품 오류가 아니라 정확히 남은 상호작용 인수 공백이다. 다음 좁은 검증은 populated 긴 이름 행의 keyboard 경로 공개와 selected/focus 동시 상태를 양테마 800/200에서 직접 관측하는 것이다. 물리 pointer/IME, screen reader 및 전체 상태 matrix는 미인수.

## 핀
직접 본 PNG: `docs/validation/evidence/windows-msvc-library-empty-pagination-private-20261012/gui/first/library-empty-initial/reach-800-200/after.png` SHA256 `ffabf51166799d0ae2167f3ebe5027d8fa508c696dd2af5babfbe069056d2ff5`
- 현재 읽은 `crates/app/src/accessible_metrics.rs` SHA256 `e0384730f03e114a4b3c9db1304ecaa1149e96770abdcbd19818e35a6ffae539`
- 현재 읽은 `crates/app/src/accessible_library.rs` SHA256 `7c1666939d3cb9e9ac7a5867d7cabe5d425e403a775c2fc4b024d274f1ed7e28`
- 현재 읽은 `crates/app/src/accessible_tools.rs` SHA256 `c7ce4ffb8486a17d0cebcf2143bab98dbd8ed1b68d7277cd4cafd8aad99ffab9`
