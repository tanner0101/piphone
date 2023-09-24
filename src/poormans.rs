use biquad::Biquad;
use biquad::ToHertz;

pub struct Filter {
    lp: biquad::DirectForm1<f32>,
    hp: biquad::DirectForm1<f32>,
}

impl Filter {
    pub fn new(sample_rate: biquad::Hertz<f32>) -> Self {
        let cutoff_frequency = 3.khz();
        let lp_coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::LowPass, 
            sample_rate, 
            cutoff_frequency, 
            biquad::Q_BUTTERWORTH_F32).unwrap();

        let hp_coeffs = biquad::Coefficients::<f32>::from_params(
            biquad::Type::HighPass, 
            sample_rate, 
            120.hz(), 
            biquad::Q_BUTTERWORTH_F32).unwrap();

        return Filter {
            lp: biquad::DirectForm1::<f32>::new(lp_coeffs),
            hp: biquad::DirectForm1::<f32>::new(hp_coeffs),
        }
    }

    pub fn run(&mut self, x: i16) -> i16 {
        return self.hp.run(self.lp.run(x as f32)).round() as i16;
    }
}
