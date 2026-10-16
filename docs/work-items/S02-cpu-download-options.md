# S02 CPU typed download options port contract

- 상태: completed
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R04, R05, R07, R08
- 담당/수정 범위: guardian_fault — 승인 후 신규 crates/app/src/accessible_options.rs 및 해당 모듈 내부 테스트만; 현재는 이 전용 문서만
- 공유 담당: root — accessible.rs/CPUmain 모듈 선언/catalog/Cargo 및 공통 검증
- 선행 작업: core/options.rs, Iced main.rs options/advanced_controls/subtitle_controls, docs/REQUIREMENTS.md R01/R04/R07 및 docs/INTEGRATIONS.md 읽기 완료. root CPU settings GUI 종료와 신규 제품 모듈 작성 승인 전 source/Cargo/build/GUI 수정 없음

## 사용자 결과

CPU 네이티브 다운로드 화면에서 최고 품질/높이 제한/오디오, 구간과 빠른·정확 자르기, 자막 언어·자동 생성 허용, 도움말 있는 주요 숫자 옵션을 선택하고 기존 core 정책으로 실행 전 검증한다. 잘못된 입력은 작업/Attempt/프로세스 생성 전에 거절한다. 이미 저장된 재개 작업은 원래 typed snapshot을 사용한다.

## 공유 인터페이스 제안

```rust
pub(crate) struct DownloadForm { /* private finite fields */ }
impl Default for DownloadForm { /* Best, Fast, empty range, no subtitles, empty advanced */ }
impl DownloadForm {
    pub(crate) fn show(&mut self, ui: &mut eframe::egui::Ui, enabled: bool);
    pub(crate) fn build(
        &self,
        url: &str,
        output: &std::path::Path,
        ffmpeg: Option<&std::path::Path>,
    ) -> Result<dlpad_core::options::DownloadOptions,
                dlpad_core::options::ValidationError>;
}
```

root가 URL/저장 위치/도구·FFmpeg 입력을 계속 소유한다. 빈 FFmpeg 입력을 None으로 변환해 넘기며 nonemptyUnicode 절대 경로의 정책은 core plan이 검증한다. build는 SourceUrl::parse → 선택 품질/TimeRange/SubtitleOptions/AdvancedOption 구성 → DownloadOptions::plan 검증 → typed options 반환만 한다. 파일/네트워크/프로세스 I/O나 shell parsing은 하지 않는다. output은 core의 nonempty 검증을 재사용하며 절대경로·staging/source ownership 검증은 기존 repo 책임이다. module에서 도구 발견/버전 검사를 중복하지 않는다.

DownloadForm 내부 상태: 명시 quality choice(Best/Height/Audio), height String, start/end String, accurate bool, subtitles Option<SubtitleOptions>, advanced_values [String;3], advanced_open bool. Height의 초기 표시는1080 제안이나 default choice는Best이므로 사용자가 선택하기 전 높이 제한을 추가하지 않는다. 기존 Iced의 숫자0=Best 표현과 달리 선택값을 명확히 분리하고 Audio를 기존 core Quality::Audio로 제공한다. 오디오 형식 변환/codec 선택을 구현했다고 표현하지 않는다.

## 입력·정책·도움말 계약

- Height 숫자 field 최대5chars; u16 파싱 오류는 root catalog QUALITY_NUMBER, 허용144..4320은 core plan 그대로. 공백/음수/소수/overflow/0 높이 선택은 거절, 별도 Best 선택으로 최고 품질을 표현한다.
- 구간 field 각32chars, 비활성 기본 empty/empty=None. 한쪽만 입력하면 TimeRange::parse_time의 기존 오류를 반환한다. 초/MM:SS/HH:MM:SS와 소수1~3자리, 분·초<60, checked arithmetic 및 end>start를 core가 판정한다. trim/자동 순서교체/반올림하지 않는다. Fast default, Accurate 명시 선택이며 기존 CUT_COST 도움말을 사용한다. 정확 자르기는 시간/재인코딩 비용 설명, 전체 다운로드는 cut argv 없음. core fast→download-sections, accurate→force-keyframes-at-cuts를 그대로 유지한다.
- 자막 None(default)/한국어/영어/한·영은 SubtitleLanguage::ALL; auto_generated 기본false, 자막None에서는자동생성 선택을 비활성화한다. None을 거쳐 다시 언어 선택할 때 false로 시작한다. 전체 영상에서만 지원하며 구간+자막 선택은 core plan의 기존 충돌 오류로 시작 거절한다. 자막 삽입/원본 언어 자동 선택/rawsub-option 추가 없음.
- 고급 옵션은 core CATALOG의 정확3keys만 사용: --limit-rate(1..1000000000000 B/s), --retries(0..20), --concurrent-fragments(1..16). 값마다 최대20chars; 빈 값은 override 미생성. 문자열은 그대로 core AdvancedOption::validate에 전달하므로 ASCII숫자/byte길이/overflow/범위 검증을 재사용한다. key는 사용자입력이 아닌 고정catalog; shell/quote/exec/rawformat/postprocessor/path/config/plugin/duplicates override를 허용하지 않는다. core plan이 retries 기본3을 override로 한 번만 남기는 계약 유지.
- 모든 노출 옵션은 summary/example/default/tradeoff/conflicts/version(unverified)을 보여준다. 고급 옵션은 core CATALOG, 자막은 SUBTITLE_HELP에서 가져온다. 품질·구간 설명은 root가 중앙catalog의 기존 QUALITY/RANGE/CUT_COST와 필요한 선택라벨만 조정한다. 새로운 UI문자열을 모듈 안에 산재시키지 않는다. 숫자 오류는 해당 입력 가까이 표시하고 build도 재검증한다.
- egui char_limit은 byte limit이 아니다. 각field char상한은 메모리 상한용이며 core byte검증을 통과 기준으로 사용한다. field를 조용히 잘라 유효 옵션으로 바꾸지 않는다. enabled=false일 때 옵션 상태 변경 불가; root가 다운로드/library/diagnostic pending/CloseWait 등 기존 active조건을 전달한다. 도움말 펼침도 disabled 그룹에 포함할지 root UX와 일치시킨다.
- URL8192byte/FFmpeg32768byte 정책은 core 재사용; root기존 field 메모리 상한은 유지. 이 모듈 추가는 font/GPU/웹런타임/의존성/새 서버를 추가하지 않는다.

## 인수 조건

1. unit: default full plan에 Best/no cut/no subtitle/no advancedoverride; Audio finiteformat, Height144/4320 valid 및143/4321/0/overflow invalid. TimeRange 소수/8h 값/partial/invalid bounds/overflow, Fast vs Accurate argv 차이 검증.
2. unit: subtitle3언어+auto=true/false typedflags, range/subtitle conflict exact거절. advanced empty/nooverride·retries0/20한 번·21invalid·limit0/overflowinvalid·fragments16valid17invalid·공백/음수/비ASCII invalid. no arbitrarykeys interface, 기존 core tests를 복제하지 않고 form→typedplan 경계를 검증한다.
3. root 통합: build가 startup 이전 정책오류를 반환하여 DB/Attempt/프로세스 미생성; 저장 snapshot의 Quality/Range/Advanced/Subtitles 일치. 실행 중 typed controls 불변, 재개는 원래 snapshot 사용. managed/tool version/실제 FFmpeg 검증은 기존 엔진 경로 유지.
4. 실제 GUI: owned Xvfb/privatebus/AT-SPI bounds와 합성tool argv fixture로 Best/Height/Audio, Fast/Accurate 구간, 자막·advanced/충돌 시작거절을 확인한다. Rust unit pass를 actualGUI/FFmpeg/subtitle 결과로 확대하지 않는다. Windows는별도frozennative exe로 actualprivateGUI 인수를 수행할 때만PASS. 사용자 display/Documents/영상/계정사용 금지, 기존8h재시작금지.

## 현재 분석 결론·남은 승인

Iced main.rs는현재숫자품질 Best/Height만구성하며core에Audio가이미있다. CPU현재options()는Best/time_rangeNone/advancedempty/subtitlesNone로 제한돼있으므로 신모듈연결이필요하다. TimeRange·advanced·subtitle validator/argv/snapshot은 이미core에구현돼 있고 새schema/Cargo/library 없이재사용할수있다. root는후속settings GUI끝난뒤 신규module작성과공유호출부를조정한다. 이문서작성단계에서 구현·unit·GUI 통과를 주장하지 않는다.

## 신규 모듈 작성

root settings GUI 종료 후 승인받아 accessible_options.rs 작성. TextEdit min_size(0,32logicalpx), 전체disabled그룹, coreplan/parse/validator/snapshot 재사용. 6개 form→plan 경계 테스트 포함. 공유catalog추가/모듈연결/rootbuild전 Cargo실행없음; rustfmt 신규파일만 완료. 아직compile/test/actualGUI미검증.

rootレビュー後、quality()共通helper→新core Quality::validateをbuild/inline双方再利用。144..4320境界のUI近傍エラーも同一policy。QUALITY_HEIGHT_NUMBERとQUALITY_HELPをcentralcatalogから参照。Audioラベルはオーディオ優先、bestaudio/best fallbackで動画の可能性・形式変換なしを説明。module単独format/sourcefreeze。core/catalog追加はroot担当、実際compiletestは統合後。

root통합44CPUtests/clippy/release 통과보고 및 실제LinuxGUI11기존cases 인수 완료. 최초2clip하네스실패 보존/각1회교정 재검증, scope cpu-options-validation.md참조. Windows/전체미디어실증은후속으로남기며사용자pause지시에따라추가작업없음.
