# S00: UI 정렬과 시각 밀도 통합

- 상태: implemented
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R11
- 담당/수정 범위: root app/accessible.rs, media_workspace.rs, ui_components.rs, ui_theme.rs, accessible_options.rs, path_picker.rs, catalog.rs, DESIGN.md, 검증/안내; diagnostic_cache는 /tmp 읽기 전용 GTMUX 비교
- 선행 작업: 미디어 작업면906 후보, 사용자 UI 재피드백

## 사용자 결과

입력·미리보기·저장 설정의 기준선, 버튼/입력 높이, 여백과 보조 정보 밀도를 통일한다. 기능을 늘리는 대신 실제 화면의 정돈을 개선한다.

## 계약과 범위

Rust native 유지. 공유 schema/Cargo/lock 변화 없음. 원 미디어/복구/취소 정책 유지. 정상 Git 저장소가 없어 branch 생성하지 않고 기존 .git을 보존한다. 별도 도구 공급 연결 작업과 파일 담당을 나눈다.

## 인수 조건

Windows private GUI 800/960 및 앱150/200, Light/Dark의 스크롤·focus 도달·저장 폴더 행과 actual PNG를 root가 검토한다. 전체 fmt/clippy/test/release 검사를 실행한다. 공개 데스크톱 fallback하지 않는다.

## 검증과 handoff

진행 중. 이전906 검증을 새 UI 완료로 재사용하지 않는다.

구현: 탐색·작업면·footer의16px기준선,32px입력/버튼,24px2열gap와300px저장inspector를공통화했다. 품질선택은좁은열에서wrap하지않는fullwidthComboBox,emptypreview는공통색/벡터play표시,타임라인은56px로축소했다. 제품문구는중앙catalog로유지했다. Windows whole native-toggle934/0/16의4명령PASS. 실제새layoutPNG와사용자시각인수는아직대기이며이전63PNG를완료증거로대체하지않는다. 도구실제시험의newtool0-usedPNG를root직접열어설정열/32px행과상태footer를검토했다.

최종전체Windows검사934/0/16,610source/4processnatural0를root검토했다. 새layout63PNG의Light→Dark→restart,800/960·앱150/200과32px경로행/도달성/아이콘handle/한글readback,2appnatural0와readonlythemeDark를root+독립검토했다. root3PNG직접view/별담당3PNG비평;URL상단scroll복귀·준비안내중복·빈여백/설정밀도·좁은저장경로는후속이다. actualsyntheticFFmpegseek/cancel/originalpreserve와RawInput19PNG는PASS,root2PNGview. URL최초staleUIA FAIL은보존하고같은40초poll한도안에해당예외만재조회한관측기로retryPASS:원본2258B/Range2·4516B/빨강→파랑·natural0·ownedserverdrain. rootactual2PNG직접view. 물리mouse/IME/OSDPI/실제사이트와사용자최종시각인수는미검증이다. 비용검사와새ZIP은진행중이다.

사용자결과와후속은 [현대 UI 비교](../development/modern-ui-audit.md)에모았다. 이작업의정렬/제어크기/시각밀도구현은인수했지만사용자디자인만족은대기한다. 공유schema/Cargo/lock변경없음. 앱정적binary는9,404,416B(이전906후보9,205,760B대비198,656B증가),설치공급연결에따른runtime/크기는별도S03작업에기록했다. 비용3회와패키지검증은마지막절차다.

최종 비용: `windows_msvc_whole_gui_cost_probe.py ...ui-polish-tool-final-20261008/result.json ...ui-polish-cost-final-20261008 --no-concurrent-dlpad-probes` PASS,3회60초·18개고정구간 모두CPU<1%/WS<80MiB. root산식재계산 최대0.785270467%·73,871,360B(70.449MiB),3DBreadonly integrity/읽기전후SHA보존확인. 외부hostquiet/재생성능은미검증. [비용root검토](../validation/evidence/windows-msvc-ui-polish-cost-final-20261008/root-review.json).

최종graph258과3binary를현재whole/lock에결합해 [개발ZIP](../validation/evidence/windows-msvc-ui-polish-graph-final-20261008/dlpad-user-preview.zip)을생성했다.17,228,708B/466members/SHA384c9108656d1cc1397563b554c6d4310f4fd309e2eb32fa451263fda86eaf80. rootCRC/3binarySHA·PE·권한/258noticeSHA/wholegraphlockpin을검토했다. selectednotice빈목록,wholelockappendix미공지43과release/legalfalse를유지했다. [실행안내](../development/windows-user-preview.md).

비용과 ZIP의 별도 독립 검토도 완료했다. 비용 3만 합성 파일/3DB/18창, ZIP 466항목 CRC/SHA·PE·권한/258notice와 현재 source·lock을 대조해 불일치 0이다. 사용자 최종 시각 평가와 위 후속은 그대로 남는다.
