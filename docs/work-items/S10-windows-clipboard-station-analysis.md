# S10: Windows 클립보드 격리 station 조사

- 상태: done
- 분류: investigation
- 우선순위: P0
- 요구사항: R13, R11, S10
- 담당/수정 범위: library_identity — 신규 전용 workitem/validation만
- 선행 작업: Windows privateDesktop는WinSta0clipboard공유 제약

## 사용자 결과

사용자 clipboard를읽거나쓰지않고P0copy를실제검증할Windows owned noninteractive windowstation 실험계약을찾는다.

## 계약과 범위

Microsoft primary문서와현재probe읽기전용. GUI/클립보드/프로세스station변경실행·제품source수정금지. 새로운비interactive station/Desktop/child lpDesktop·rights·cleanup·AccessKit제약을조사하고미실험을명시한다.

## 인수 조건

clipboard station격리공식근거,SetProcessWindowStation processscope,childinheritance/accessrights/보안/cleanup/reader제약과bounded실험가능성구분. 별도admin/user/system변경필요시해당부분만blocked기록.

## 검증과 handoff

primary자료와기존probe read-only 조사완료. docs/validation/windows-clipboard-station-analysis.md에권한·station생성·child·cleanup·UIA한계·bounded실험인수를정리했다. 실제API/GUI/clipboard실험없음, WindowsP0copy gate는아직미검증.
